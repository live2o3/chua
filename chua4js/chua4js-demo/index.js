import * as wasm from "chua4js";

export async function upload(baseUrl, file, chunkSize, parallel) {
    return await wasm.upload(baseUrl, file, chunkSize, parallel);
}
