package com.live2o3;

import java.util.UUID;

public class Chua {

    static {
        System.loadLibrary("chua4j");
    }

    public static native Result<UUID> upload(String baseUrl, String path, long chunkSize, int parallel);
}