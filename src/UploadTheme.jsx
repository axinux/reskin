import React, { useState } from "react";
import { Client, Storage, Databases, Account, Query } from "appwrite";
import { invoke } from "@tauri-apps/api/core";

export default function UploadTheme({ onNavigate }) {
  const [file, setFile] = useState(null);
  const [themeInfo, setThemeInfo] = useState(null);
  const [status, setStatus] = useState("Pick a .reskin file");
  const [loading, setLoading] = useState(false);

  const client = new Client()
    .setEndpoint(import.meta.env.VITE_APPWRITE_ENDPOINT)
    .setProject("reskin");
  const storage = new Storage(client);
  const databases = new Databases(client);
  const account = new Account(client);

  const bucketId = "themes";
  const databaseId = "reskin";
  const collectionId = "themes";

  const showStatus = (msg) => setStatus(msg);

  const getUser = async () => {
    let user = JSON.parse(localStorage.getItem("reskin_user"));
    if (user && user.$id) return user;
    try {
      user = await account.get();
      localStorage.setItem("reskin_user", JSON.stringify(user));
      return user;
    } catch {
      return null;
    }
  };

  const getFileHash = async (file) => {
    const arrayBuffer = await file.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, "0")).join("");
  };

  const handleFileChange = async (e) => {
    const f = e.target.files[0];
    if (!f) return;
    setFile(f);
    showStatus("Reading theme metadata...");
    try {
      const arrayBuffer = await f.arrayBuffer();
      const uint8Array = Array.from(new Uint8Array(arrayBuffer));
      const info = await invoke("extract_theme_info", { fileData: uint8Array });
      const safeTags = typeof info.tags === "string"
        ? info.tags.slice(0, 32)
        : Array.isArray(info.tags)
          ? info.tags.join(",").slice(0, 32)
          : "";
      setThemeInfo({ ...info, tags: safeTags });
      showStatus("Metadata loaded! Ready to upload");
    } catch (err) {
      console.error(err);
      showStatus("Failed to read theme metadata");
    }
  };

  const handleUpload = async () => {
    if (!file || !themeInfo) return;
    setLoading(true);
    showStatus("Checking login...");

    const user = await getUser();
    if (!user) {
      showStatus("Not logged in. Please log in first!");
      setLoading(false);
      return;
    }

    showStatus("Checking for duplicates...");
    try {
      const hash = await getFileHash(file);

      const existing = await databases.listDocuments(databaseId, collectionId, [
        Query.equal("hash", hash)
      ]);

      if (existing.total > 0) {
        showStatus("This theme file already exists!");
        setLoading(false);
        return;
      }

      showStatus("Uploading file...");
      const uploadedFile = await storage.createFile(bucketId, "unique()", file);
      const fileId = uploadedFile.$id;

      showStatus("Saving metadata...");
      const docData = {
        name: themeInfo.name,
        description: themeInfo.description || "",
        author: themeInfo.author || "Unknown",
        version: themeInfo.version || "",
        tags: themeInfo.tags || "",
        license: themeInfo.license || "MIT",
        file: fileId,
        hash,
      };

      const permissions = [
        `read("user:${user.$id}")`,
        `write("user:${user.$id}")`
      ];

      await databases.createDocument(databaseId, collectionId, "unique()", docData, permissions, permissions);

      showStatus("Upload successful!");
      onNavigate?.("marketplace");
    } catch (err) {
      console.error(err);
      showStatus(`Upload failed: ${err.message || err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h1>📤 Upload Theme</h1>
      <input type="file" accept=".reskin" onChange={handleFileChange} />
      {themeInfo && (
        <div style={{ marginTop: "1rem" }}>
          <h3>Preview:</h3>
          <p><strong>Name:</strong> {themeInfo.name}</p>
          <p><strong>Author:</strong> {themeInfo.author}</p>
          <p><strong>Description:</strong> {themeInfo.description}</p>
          <p><strong>Version:</strong> {themeInfo.version}</p>
          <p><strong>Tags:</strong> {themeInfo.tags}</p>
          <p><strong>License:</strong> {themeInfo.license}</p>
        </div>
      )}
      <button disabled={!file || !themeInfo || loading} onClick={handleUpload}>
        {loading ? "Uploading..." : "Upload Theme"}
      </button>
      <p>{status}</p>
    </div>
  );
}
