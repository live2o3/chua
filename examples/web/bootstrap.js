// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("./index.js").then(m => {
    document.getElementById("baseUrl").value = window.location.protocol + "//" + window.location.host;
    document.getElementById("upload").onclick = async function () {
        let start = new Date();
        let baseUrl = document.getElementById("baseUrl").value;
        let file = document.getElementById("path").files[0];
        let chunkSize = parseInt(document.getElementById("chunkSize").value);
        let parallel = parseInt(document.getElementById("parallel").value);
        console.log("upload => baseUrl: " + baseUrl + ", file: " + file.name + ", chunkSize: " + chunkSize + " Bytes, parallel: " + parallel + ".");
        try {
            let fileId = await m.upload(baseUrl, file, chunkSize, parallel);
            console.log("File uploaded.(fileId: " + fileId + ", duration: " + (new Date() - start) + "ms.)");
        } catch (e) {
            console.error("Failed to upload file: " + e);
        }
    };
})
  .catch(e => console.error("Error importing `index.js`:", e));
