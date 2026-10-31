# PdfCraft ฉบับปรับแต่งของเรา

Branch `custom/thai-no-ads` เพิ่มสามเรื่องจากต้นทาง ([storytold/pdfcraft](https://github.com/storytold/pdfcraft)):

1. อ่านและเขียนภาษาไทยด้วย font ที่มีในเครื่อง ไม่เป็น `[]`
2. เอาการ์ด "Join the ArtCraft community" บนหน้า Home และปุ่ม Discord บนแถบด้านบนออก
3. ติดป้าย TH บนไอคอนแอป เพื่อให้รู้ว่าเป็นเวอร์ชันไทย

**สถานะ (2026-10-31):** build และทดสอบผ่านบน macOS แล้ว ยังไม่เคย build หรือทดสอบบน Windows

## กติกา

- `main` ต้องตรงกับต้นทางเสมอ ห้าม commit งานของเราบน `main`
- งานปรับแต่งทั้งหมดอยู่บน `custom/thai-no-ads`
- ถ้าหน้า GitHub ขึ้นปุ่ม **Discard commits** ห้ามกด
- ใช้แอปที่ build จาก branch นี้เท่านั้น ปุ่มเช็กอัปเดตในแอปชี้ไปที่ release ของต้นทาง ซึ่งไม่มีของที่เราแก้
- ไม่ส่ง branch นี้เป็น PR ไปต้นทาง เพราะการฝัง font ที่ติดตั้งในเครื่องลง PDF ขัดกับ `AGENTS.md`

## เวลาต้นทางมี update

1. กด **Sync fork** บนหน้า GitHub ของ fork ตอนที่เลือก branch `main` อยู่
2. commit งานที่ค้างอยู่บน `custom/thai-no-ads` ให้เรียบร้อย
3. ดึง `main` ลงเครื่อง แล้ว merge เข้า branch ของเรา:

```bash
git switch main
```

```bash
git pull --ff-only origin main
```

```bash
git switch custom/thai-no-ads
```

```bash
git merge main
```

ถ้า `--ff-only` ไม่ผ่าน แปลว่ามี commit อื่นอยู่บน `main` ให้หยุดดูก่อนว่าเป็น commit อะไร

4. ถ้ามี conflict ให้แก้ตามตารางด้านล่าง แล้วรัน:

```bash
git add -A
```

```bash
git commit --no-edit
```

5. ตรวจว่าของที่เราแก้ยังอยู่ครบ และของที่ลบไปไม่กลับมา ขั้นนี้ไม่ต้อง build และเสร็จทันที:

```bash
python3 packaging/custom-check.py
```

merge ผ่านไม่ได้แปลว่างานเรายังอยู่ เพราะต้นทางอาจเพิ่มการ์ดโปรโมทใหม่หรือเปลี่ยนโค้ดที่เราเรียกใช้โดยไม่เกิด conflict ถ้าสคริปต์ไม่ผ่าน มันจะบอกไฟล์และเรื่องที่ผิด ให้แก้ตามตารางด้านล่างแล้วรันซ้ำ

6. build และตรวจตามหัวข้อ "ตรวจหลัง merge"
7. push ขึ้น GitHub:

```bash
git push origin custom/thai-no-ads
```

## จุดที่เราแก้ และวิธีแก้ถ้าโดน conflict

| ไฟล์ | สิ่งที่แก้ | ถ้าโดน conflict |
|---|---|---|
| `crates/fonts/src/craft.rs` | `document_thai_font` ถอยมาใช้ `installed_thai_font` และ `standalone_face` แยก face ออกจาก `.ttc` | เก็บทั้งคู่ไว้ เพราะ shaping และการเขียนลง PDF ไม่รับ `.ttc` โดยตรง ต้องเลือกจากชื่อเต็ม "... Regular" เพราะ Thin กับ Medium ก็เรียกตัวเองว่า Regular |
| `crates/fonts/src/lib.rs` | export `installed_thai_font` | เก็บชื่อนี้ไว้ในรายการ |
| `crates/ui-egui/src/platform/system_fonts.rs` | `thai_fallback`, `thai_candidates`, `thai_face`, `read_face` | เก็บลำดับ font ของเรา ห้ามใส่ Thonburi เพราะสระจะมีวงกลมประติดมา และต้องเลือก face "Text" ของ Sukhumvit Set ไม่ใช่ Thin |
| `crates/ui-egui/src/theme.rs` | `SYSTEM_THAI_FALLBACK` ในรายการ font ของเครื่อง | เก็บไว้ ถ้าต้นทางเพิ่มของเหมือนกันให้เหลือชุดเดียว |
| `crates/edit/src/text_regions/styled.rs` | `thai_face` ใน `FontResolver`, การเลือก font ใน `prepare`, ท้าย `shape_source` | ต้องเก็บทั้งสองจุด: `prepare` ใช้กับตัวอักษรที่ font ใน PDF ไม่มี ส่วน `shape_source` ใช้กับ font ที่ไม่มี GSUB |
| `crates/render/src/system_font.rs`, `crates/render/src/raster.rs` | `Script::Thai` และ `names_thai_family` | เก็บไว้ ใช้กับ PDF ที่ไม่ฝัง font เฉพาะตอนแสดงบนจอ ห้ามเอา Tahoma ขึ้นก่อน เพราะไม่มี shaping แล้ววรรณยุกต์จะหาย |
| `crates/ui-egui/src/home.rs` | ลบการ์ด community | ลบต่อไป แม้ต้นทางจะแก้ข้อความในการ์ด |
| `crates/ui-egui/src/chrome.rs` | ลบปุ่ม Discord และความกว้างที่กันไว้ให้ปุ่ม | ลบต่อไป `controls_width` ต้องไม่นับปุ่ม Discord |
| `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/app.rs` | ลบ field `community_card_closed` | ลบต่อไป |

test ที่เราแก้ตามไปด้วย: `crates/edit/src/tests.rs` และ `crates/ui-egui/tests/it/` (`content.rs`, `editing.rs`, `fonts.rs`, `links.rs`, `ui.rs`)

ถ้าต้นทาง merge [PR #5132](https://github.com/storytold/pdfcraft/pull/5132) จะมี Thai fallback ซ้ำกับของเรา ให้เหลือชุดเดียว และคงลำดับ font กับ `thai_face` ของเราไว้

ถ้าต้นทางเพิ่มการ์ดหรือปุ่มโปรโมทใหม่ git จะไม่เอาออกให้ ต้องลบเองแล้วเพิ่มลงตารางนี้

ถ้าไฟล์ใน `assets/app-icon/` หรือ `ATTRIBUTION.toml` โดน conflict ให้เอาของต้นทางมา แล้วติดป้าย TH ใหม่:

```bash
git checkout main -- assets/app-icon ATTRIBUTION.toml
```

```bash
python3 packaging/thai-badge.py
```

```bash
cargo xtask assets --write
```

ต้องใช้ Pillow ของ Python และรันบน Mac เพื่อให้ได้ไฟล์ `.icns`

## font ที่ใช้

- **ตัวแอป:** Sukhumvit Set, Sathu, Silom, Tahoma ตามลำดับ
- **เปิดดู PDF ที่ไม่ฝัง font:** Sukhumvit Set, Sathu, Tahoma ตามลำดับ
- **เขียนลง PDF บน macOS:** ไฟล์ที่ผู้ใช้ติดตั้งเอง (`Sarabun-Regular.ttf`, `THSarabunNew.ttf`, `NotoSansThai-Regular.ttf`), `Sarabun.ttc` ของ macOS, TH SarabunPSK ของ Word แล้วจึงเป็น Tahoma
- **เขียนลง PDF บน Windows:** ไฟล์ที่ติดตั้งเองใน `C:\Windows\Fonts`, Leelawadee UI แล้วจึงเป็น Tahoma
- **ถ้า build พร้อม `CRAFT_FONTS_DIR`:** font ไทยจาก craft-fonts จะถูกใช้ก่อน font ในเครื่อง
- ตอนนี้โปรแกรมเลือก font ให้อัตโนมัติ ยังไม่มีเมนูให้ผู้ใช้เลือก font ไทยเอง
- `PDFCRAFT_SYSTEM_FONTS=0` ปิดการอ่าน font ในเครื่องทั้งหมด

## ตรวจหลัง merge

Homebrew ไม่ได้เพิ่ม `cargo` เข้า PATH ให้ ถ้า terminal หาไม่เจอให้รันบรรทัดนี้ก่อน:

```bash
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
```

รัน test เฉพาะ crate ที่เราแก้ ห้ามใช้ `--workspace` เพราะหนักเกินไปสำหรับเครื่อง RAM 8 GB:

```bash
cargo test -p pdfcraft-fonts -j 2
```

```bash
cargo test -p pdfcraft-render -j 2
```

```bash
cargo test -p pdfcraft-edit -j 2
```

```bash
cargo test -p pdfcraft-ui-egui -j 2
```

test `editing::a_custom_property_whose_name_the_tools_refuse_is_listed_and_left_alone` ล้มอยู่แล้วบน `main` เมื่อ build โดยไม่มี `CRAFT_FONTS_DIR` ไม่เกี่ยวกับงานเรา ถ้ามี test อื่นล้มให้ลองเทียบกับ `main` ก่อนเสมอ

ถ้าเทียบผ่าน git worktree ห้ามใช้โฟลเดอร์ `target` ร่วมกับ branch นี้ เพราะ build script ของ `pdfcraft-ui-egui` จะจำ path ของ worktree ไว้ พอลบ worktree แล้วรายชื่อในหน้า About จะว่างและ test `about_dialog_has_contributors_and_models_tabs` จะล้ม แก้ได้ด้วย `cargo clean -p pdfcraft-ui-egui`

เปิดแอปด้วย:

```bash
cargo run -p pdfcraft -- ไฟล์.pdf
```

ลองในแอปจริง:

- [ ] ชื่อไฟล์ไทยในแท็บและ Recent ไม่เป็นสี่เหลี่ยม
- [ ] พิมพ์ไทยในช่องค้นหาและช่องข้อความได้
- [ ] คำว่า `ที่ น้ำ ผู้ใช้ กิ่` วรรณยุกต์ไม่ซ้อนสระ
- [ ] ไอคอนแอปมีป้าย TH
- [ ] Add Text พิมพ์ไทย บันทึก เปิดใหม่ แล้วคัดลอกข้อความได้
- [ ] Edit Text แก้ย่อหน้าใน PDF ไทย บันทึก แล้วเปิดในโปรแกรมอื่นได้
- [ ] PDF ไทยที่ไม่ฝัง font แสดงบนจอได้
- [ ] หน้า Home ไม่มีการ์ด community และแถบด้านบนไม่มีปุ่ม Discord

## ยังไม่รองรับ

- Windows ยังไม่เคย build หรือทดสอบ และลำดับ font ของ Windows ยังไม่ได้ดูผลจริง
- comment และ form field ที่พิมพ์ไทยยังเป็น `?`
- ตัวหนาและตัวเอียงตอนเขียนไทยลง PDF: ใช้ face Regular อย่างเดียว
- Edit Text กับข้อความที่หมุนอยู่ หรือการเปลี่ยนตัวหนาหรือ font พร้อมกับพิมพ์ไทย จะขึ้น error
- หน้า PDF ที่ไม่ฝัง font แก้เฉพาะตอนดูบนจอ ไม่รวมตอนพิมพ์หรือ export และต้องเป็น font ชนิด CID ที่ชื่อเป็น font ไทย
- ลิงก์ Discord ใน Help menu และหน้า About ยังอยู่ เพราะไม่ใช่โฆษณา
