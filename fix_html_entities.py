#!/usr/bin/env python3
import os
import glob

# Znajdź wszystkie pliki .rs
rs_files = glob.glob('src/**/*.rs', recursive=True)

count = 0
for file_path in rs_files:
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Usuń HTML entities
    original_content = content
    content = content.replace('&amp;', '&')
    content = content.replace('&lt;', '<')
    content = content.replace('&gt;', '>')
    content = content.replace('&quot;', '"')
    content = content.replace('&#39;', "'")
    
    # Zapisz tylko jeśli zmieniono
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        count += 1
        print(f"Zaktualizowano: {file_path}")

print(f"\nZakończono. Zaktualizowano {count} plików.")