#!/bin/bash

# Script optimizado para iniciar el servicio Java rápidamente

JAVA_OPTS="-XX:+UseG1GC \
           -XX:+UnlockExperimentalVMOptions \
           -XX:+UseJVMCICompiler \
           -Xms128m \
           -Xmx512m \
           -XX:+TieredCompilation \
           -XX:TieredStopAtLevel=1 \
           -XX:+UseStringDeduplication \
           -XX:+OptimizeStringConcat \
           -Djava.security.egd=file:/dev/./urandom \
           -Dspring.main.lazy-initialization=true \
           -Dspring.jmx.enabled=false"

# Buscar el JAR automáticamente (más flexible)
JAR_FILE=$(find target -name "*.jar" -not -name "*-sources.jar" -not -name "*-javadoc.jar" | head -1)

# Si no encuentra ningún JAR, usar el nombre por defecto como fallback
if [ -z "$JAR_FILE" ]; then
    JAR_FILE="target/poker-probability-service-1.0.0.jar"
fi

# Verificar si el JAR existe
if [ ! -f "$JAR_FILE" ]; then
    echo "JAR file not found: $JAR_FILE"
    echo "Building project..."
    mvn package -DskipTests -q -T 1C
    
    # Buscar de nuevo después de compilar
    JAR_FILE=$(find target -name "*.jar" -not -name "*-sources.jar" -not -name "*-javadoc.jar" | head -1)
    
    if [ -z "$JAR_FILE" ]; then
        echo "Error: No JAR file found after compilation"
        exit 1
    fi
fi

echo "Starting Java service with JAR: $JAR_FILE"
echo "Using optimized JVM settings..."
java $JAVA_OPTS -jar "$JAR_FILE"
