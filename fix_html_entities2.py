#!/usr/bin/env python3
import os
import glob
import re

# Znajdź wszystkie pliki .rs
rs_files = glob.glob('src/**/*.rs', recursive=True)

count = 0
for file_path in rs_files:
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Usuń HTML entities używając regex
    original_content = content
    content = re.sub(r'&amp;', '&', content)
    content = re.sub(r'&lt;', '<', content)
    content = re.sub(r'&gt;', '>', content)
    content = re.sub(r'&quot;', '"', content)
    content = re.sub(r'&#39;', "'", content)
    
    # Zapisz tylko jeśli zmieniono
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        count += 1
        print(f"Zaktualizowano: {file_path}")

print(f"\nZakończono. Zaktualizowano {count} plików.")