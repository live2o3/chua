#![allow(non_snake_case)]

use chua::{upload, Exception};
use jni::objects::{JClass, JObject, JString, JThrowable, JValue};
use jni::sys::{jint, jlong, jsize, JavaVM, JNI_VERSION_1_6};
use jni::JNIEnv;
use lazy_static::lazy_static;
use log::*;
use std::ffi::c_void;
use tokio::runtime::Runtime;
use uuid::Uuid;

lazy_static! {
    static ref RUNTIME: Result<Runtime, String> = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .map_err(|e| e.to_string());
}

#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(_vm: *const JavaVM, _reserved: *const c_void) -> jint {
    #[cfg(target_os = "android")]
    init_android_log();

    info!("Chua4j loaded.");

    JNI_VERSION_1_6
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_live2o3_chua_Chua_upload<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    base_url: JString<'a>,
    path: JString<'a>,
    chunk_size: jlong,
    parallel: jsize,
) -> JObject<'a> {
    if base_url.is_null() {
        return make_java_result(env, Err("Base url must not be null".into()));
    }

    let base_url: String = match env.get_string(base_url) {
        Ok(base_url) => base_url,
        Err(e) => return make_java_result(env.clone(), Err(e.into())),
    }
    .into();

    if path.is_null() {
        return make_java_result(env, Err("Path must not be null".into()));
    }

    let path: String = match env.get_string(path) {
        Ok(path) => path,
        Err(e) => return make_java_result(env.clone(), Err(e.into())),
    }
    .into();

    let chunk_size = if chunk_size > 0 {
        chunk_size as u64
    } else {
        return make_java_result(env, Err("Chunk size must be greater than 0".into()));
    };

    let parallel = if parallel >= 0 {
        parallel as usize
    } else {
        return make_java_result(env, Err("Parallel must not be less than 0".into()));
    };

    let result = RUNTIME
        .as_ref()
        .unwrap()
        .handle()
        .block_on(upload(&base_url, path, chunk_size, parallel));

    make_java_result(env, result)
}

fn make_java_result(env: JNIEnv, result: Result<Uuid, Exception>) -> JObject {
    let class = env
        .find_class("com/live2o3/Result")
        .expect("Cannot find class 'Result'");

    let java_result = match result {
        Ok(uuid) => match env.new_string(uuid.to_string()) {
            Ok(uuid) => env
                .call_static_method(
                    class,
                    "succeed",
                    "(Ljava/lang/Object;)Lcom/live2o3/Result;",
                    &[JValue::Object(JObject::from(uuid))],
                )
                .expect("Failed to call static method 'com.live2o3.Result.succeed'"),
            Err(e) => {
                let error = env
                    .new_string(e.to_string())
                    .expect("Failed to create a string");
                let cause: JThrowable = env
                    .new_object(
                        "java/lang/Exception",
                        "(Ljava/lang/String;)V",
                        &[JValue::from(error)],
                    )
                    .expect("Failed to create a Exception object")
                    .into();
                env.call_static_method(
                    class,
                    "fail",
                    "(Ljava/lang/Throwable;)Lcom/live2o3/Result;",
                    &[JValue::Object(cause.into())],
                )
                .expect("Failed to call static method 'com.live2o3.Result.succeed'")
            }
        },
        Err(e) => {
            let error = env
                .new_string(e.to_string())
                .expect("Failed to create a string");
            let cause: JThrowable = env
                .new_object(
                    "java/lang/Exception",
                    "(Ljava/lang/String;)V",
                    &[JValue::from(error)],
                )
                .expect("Failed to create a Exception object")
                .into();
            env.call_static_method(
                class,
                "fail",
                "(Ljava/lang/Throwable;)Lcom/live2o3/Result;",
                &[JValue::Object(cause.into())],
            )
            .expect("Failed to call static method 'com.live2o3.Result.succeed'")
        }
    };

    JObject::from(
        java_result
            .l()
            .expect("Failed to unwrap 'JValue' to a Java Object."),
    )
}

#[cfg(target_os = "android")]
fn init_android_log() {
    use android_logger::Config;
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Debug) // limit log level
            .with_tag("chua"),
    );
}
