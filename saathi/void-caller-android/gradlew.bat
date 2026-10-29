@echo off
java -Dorg.gradle.appname=gradlew -classpath "%~dp0gradle\wrapper\gradle-wrapper.jar" org.gradle.wrapper.GradleWrapperMain %*
