package com.live2o3.chua;

import com.live2o3.Result;
import java.util.UUID;

public class Chua {

    static {
        System.loadLibrary("chua4j");
    }

    public static native Result<String> upload(String baseUrl, String path, long chunkSize, int parallel);

    public static native void test();
}