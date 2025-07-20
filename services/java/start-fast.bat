@echo off

set JAVA_OPTS=-XX:+UseG1GC -Xms128m -Xmx512m -XX:+TieredCompilation -XX:TieredStopAtLevel=1 -XX:+UseStringDeduplication -Djavalin.showJavalinBanner=false

REM Buscar JAR automáticamente
for %%f in (target\*.jar) do (
    if not "%%~nxf"=="*-sources*" if not "%%~nxf"=="*-javadoc*" (
        set JAR_FILE=%%f
        goto :found
    )
)

:found
if not defined JAR_FILE set JAR_FILE=target\poker-probability-service-1.0.0.jar

if not exist "%JAR_FILE%" (
    echo JAR file not found: %JAR_FILE%
    echo Building project...
    mvn package -DskipTests -q -T 1C
    
    for %%f in (target\*.jar) do (
        if not "%%~nxf"=="*-sources*" if not "%%~nxf"=="*-javadoc*" (
            set JAR_FILE=%%f
            goto :found2
        )
    )
    
    :found2
    if not defined JAR_FILE (
        echo Error: No JAR file found after compilation
        exit /b 1
    )
)

echo Starting Java service with JAR: %JAR_FILE%
echo Using optimized JVM settings...
java %JAVA_OPTS% -jar "%JAR_FILE%"