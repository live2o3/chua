#![allow(non_snake_case)]

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jlong, jsize};
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_com_live2o3_Chua_upload<'a>(
    env: JNIEnv<'a>,
    class: JClass<'a>,
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

    let class = env
        .find_class("com/live2o3/Result")
        .expect("cannot find class Result");
    let result = env
        .call_static_method(class, "ok", "sig", &[JValue::Object(JObject::null())])
        .expect("wtf");

    //TODO: upload

    result.l().expect("")
}
