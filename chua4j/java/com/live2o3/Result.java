package com.live2o3;

public class Result<T> {

    private final boolean succeeded;
    private final T value;
    private final Throwable cause;

    static {
        System.loadLibrary("chua4j");
    }

    public static <T> Result<T> succeed(T value) {
        return new Result<>(true, value, null);
    }

    public static <T> Result<T> fail(Throwable cause) {
        return new Result<>(false, null, cause);
    }

    public T value() {
        return this.value;
    }

    public Throwable cause() {
        return this.cause;
    }

    public boolean succeeded() {
        return this.succeeded;
    }

    public boolean failed() {
        return !this.succeeded;
    }

    private Result(boolean succeeded, T value, Throwable cause) {
        this.succeeded = succeeded;
        this.value = value;
        this.cause = cause;
    }
}