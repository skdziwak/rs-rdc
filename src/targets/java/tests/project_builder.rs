use crate::errors::Error;
use crate::targets::java::JavaClass;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use uuid::Uuid;

const BUILD_GRADLE: &str = r#"
plugins {
    id 'java'
    id 'application'
}

application {
    mainClass = 'com.rdc.Main'
}

group 'com.rdc'
version '1.0-SNAPSHOT'

repositories {
    mavenCentral()
}

dependencies {
    implementation group: 'com.fasterxml.jackson.core', name: 'jackson-core', version: '2.14.1'
    implementation group: 'com.fasterxml.jackson.core', name: 'jackson-databind', version: '2.14.1'

    testImplementation 'org.junit.jupiter:junit-jupiter-api:5.8.1'
    testRuntimeOnly 'org.junit.jupiter:junit-jupiter-engine:5.8.1'
}

test {
    useJUnitPlatform()
}

run {
    standardInput = System.in
}
"#;

const SETTINGS_GRADLE: &str = r#"
rootProject.name = 'gradle-test'
"#;

const UTILS_JAVA: &str = r#"
package com.rdc;

import java.io.IOException;

public class Utils {
    public static String input() {
        try {
            var stream = System.in;
            var bytes = stream.readAllBytes();
            stream.close();
            return new String(bytes, java.nio.charset.StandardCharsets.UTF_8);
        } catch (java.io.IOException ex) {
            throw new java.lang.RuntimeException(ex);
        }
    }
}
"#;

struct Gradle;
static GRADLE_MUTEX: Mutex<Gradle> = Mutex::new(Gradle);

pub fn run_java(classes: &Vec<JavaClass>, input: &str) -> Result<String, Error> {
    println!("Waiting for gradle lock");
    let guard = GRADLE_MUTEX.lock().unwrap();
    println!("Got gradle lock");
    let result = run_gradle(classes, input);
    drop(guard);
    result
}

fn run_gradle(classes: &Vec<JavaClass>, input: &str) -> Result<String, Error> {
    println!("Running Java code...");
    let temp_dir = std::env::temp_dir();
    let project_dir = temp_dir.join(format!("rdc/{}", Uuid::new_v4()));
    let src_dir = project_dir.join("src/main/java/com/rdc");
    println!("Creating project directory: {}", project_dir.display());
    std::fs::create_dir_all(&src_dir)
        .map_err(|_| Error::new("Failed to create project directory"))?;

    println!("Writing java...");
    for class in classes {
        let class_path = src_dir.join(format!("{}.java", class.name()));
        let contents = class.code();
        let contents = format!("package com.rdc;\n\n{contents}");
        std::fs::write(class_path, contents)
            .map_err(|_| Error::new("Failed to write class file"))?;
    }

    println!("Writing build.gradle...");
    let build_gradle_path = project_dir.join("build.gradle");
    std::fs::write(build_gradle_path, BUILD_GRADLE)
        .map_err(|_| Error::new("Failed to write build.gradle"))?;

    println!("Writing settings.gradle...");
    let settings_gradle_path = project_dir.join("settings.gradle");
    std::fs::write(settings_gradle_path, SETTINGS_GRADLE)
        .map_err(|_| Error::new("Failed to write settings.gradle"))?;

    println!("Writing Utils.java...");
    let utils_java_path = src_dir.join("Utils.java");
    std::fs::write(utils_java_path, UTILS_JAVA)
        .map_err(|_| Error::new("Failed to write Utils.java"))?;

    let output = run_command(
        "gradle",
        &["-q", "run"],
        project_dir.to_str().unwrap(),
        input,
    );

    println!("Cleaning up...");
    std::fs::remove_dir_all(&project_dir)
        .map_err(|_| Error::new("Failed to remove project directory"))?;

    output
}

fn run_command(cmd: &str, args: &[&str], dir: &str, std_input: &str) -> Result<String, Error> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.current_dir(dir);

    println!("Running command: {cmd}");
    let mut child = command
        .spawn()
        .map_err(|_| Error::new("Failed to run command"))?;

    println!("Writing to stdin...");
    let stdin = child
        .stdin
        .as_mut()
        .ok_or(Error::new("Failed to get stdin"))?;
    stdin
        .write_all(std_input.as_bytes())
        .map_err(|_| Error::new("Failed to write to stdin"))?;

    println!("Waiting for command to finish...");
    let output = child
        .wait_with_output()
        .map_err(|_| Error::new("Failed to wait for command"))?;

    let stdout = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
    let stderr = String::from_utf8_lossy(output.stderr.as_slice()).to_string();
    if !stderr.is_empty() {
        println!("stderr: {stderr}");
        Err(Error::new("Command failed"))
    } else {
        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use crate::targets::java::JavaClass;

    #[test]
    fn run_java_test() {
        let classes = vec![JavaClass::new(
            "Main".to_string(),
            r#"
                            public class Main {
                                public static void main(String[] args) throws Exception {
                                    System.out.print("Hello, World!");
                                }
                            }"#
            .to_string(),
        )];
        let result = super::run_java(&classes, "");
        if let Err(e) = &result {
            println!("{}", e.message());
        }

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn uppercase_stdin_test() {
        let classes = vec![JavaClass::new(
            "Main".to_string(),
            r#"
                            public class Main {
                                public static void main(String[] args) throws Exception {
                                    System.out.print(Utils.input().toUpperCase());
                                }
                            }"#
            .to_string(),
        )];
        let result = super::run_java(&classes, "hello, world!");
        if let Err(e) = &result {
            println!("{}", e.message());
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO, WORLD!");
    }

    #[test]
    fn jackson_test() {
        let classes = vec![JavaClass::new(
            "Main".to_string(),
            r#"
                            public class Main {

                                static class Obj {
                                   @com.fasterxml.jackson.annotation.JsonIgnore
                                   public int height;
                                }
                                public static void main(String[] args) throws Exception {
                                    System.out.print("success");
                                }
                            }"#
            .to_string(),
        )];
        let result = super::run_java(&classes, "");
        if let Err(e) = &result {
            println!("{}", e.message());
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
