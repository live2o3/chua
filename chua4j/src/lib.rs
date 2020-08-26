#![allow(non_snake_case)]

use chua::upload;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jlong, jsize, JavaVM};
use jni::JNIEnv;
use std::ffi::c_void;
use tokio::io::Error;
use tokio::runtime::Runtime;

#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(jvm: JavaVM, _reserved: *mut c_void) {
    #[cfg(target_os = "android")]
    init_android_log();

    log::info!("Chua4j loaded.");

    match tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
    {
        Ok(_) => log::info!("Tokio runtime is created."),
        Err(e) => log::error!("Failed to create a tokio runtime: {}", e),
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_live2o3_Chua_upload<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    base_url: JString<'a>,
    path: JString<'a>,
    chunk_size: jlong,
    parallel: jsize,
) -> JObject<'a> {
    let base_url: String = env
        .get_string(base_url)
        .expect("cannot get base_url")
        .into();
    let path: String = env.get_string(path).expect("cannot get path").into();

    let chunk_size = if chunk_size > 0 {
        chunk_size as u64
    } else {
        return JObject::null();
    };

    let parallel = if parallel >= 0 {
        parallel as usize
    } else {
        return JObject::null();
    };

    let class = env
        .find_class("com/live2o3/Result")
        .expect("cannot find class Result");

    let result = env
        .call_static_method(class, "ok", "sig", &[JValue::Object(JObject::null())])
        .expect("wtf");

    let handle = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(e) => return JObject::null(),
    };

    match handle.block_on(upload(&base_url, path, chunk_size, parallel)) {
        Ok(uuid) => result.l().expect(""),
        Err(e) => JObject::null(),
    }
}

#[cfg(target_os = "android")]
fn init_android_log() {
    use android_logger::{Config, FilterBuilder};
    use log::Level;
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Debug) // limit log level
            .with_tag("chua"),
    );
}
