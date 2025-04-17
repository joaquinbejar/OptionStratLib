#!/bin/bash

SRC_DIR="target/doc/optionstratlib"
DEST_DIR="doc/framework"

echo "🛠️ Convirtiendo documentación de $SRC_DIR a Markdown en $DEST_DIR"

# Recorre todos los .html recursivamente
find "$SRC_DIR" -type f -name "*.html" | while read -r html_file; do
  # Define ruta relativa y destino
  relative_path="${html_file#$SRC_DIR/}"
  md_file="$DEST_DIR/${relative_path%.html}.md"

  # Crea subdirectorios si no existen
  mkdir -p "$(dirname "$md_file")"

  # Convierte con pandoc
  pandoc "$html_file" -f html -t markdown -o "$md_file"

  echo "✅ $relative_path → ${md_file#$DEST_DIR/}"
done

echo "📄 Conversión completa. Archivos disponibles en: $DEST_DIR"