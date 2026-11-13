fn main(){
 println!("cargo:rustc-check-cfg=cfg(bmi_low_memory)");
 println!("cargo:rerun-if-env-changed=BMI_VARIANT");
 if std::env::var("BMI_VARIANT").as_deref()==Ok("low-memory"){println!("cargo:rustc-cfg=bmi_low_memory");}
}
