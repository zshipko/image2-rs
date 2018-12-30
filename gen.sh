bindgen  \
  --whitelist-function stbi_load \
  --whitelist-function stbi_load_16 \
  --whitelist-function stbi_loadf \
  --whitelist-function stbi_load_from_memory \
  --whitelist-function stbi_load_16_from_memory \
  --whitelist-function stbi_loadf_from_memory \
  --whitelist-function stbi_write_png \
  --whitelist-function stbi_write_jpg \
  --whitelist-function stbi_write_tga \
  --whitelist-function stbi_write_hdr \
  --raw-line  "#![allow(non_camel_case_types)]" \
  stb/stb.c > src/io/stb.rs
