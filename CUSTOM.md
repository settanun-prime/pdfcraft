# PdfCraft ฉบับปรับแต่งของเรา

Branch `custom/thai-no-ads` เพิ่มสองเรื่องจากต้นทาง ([storytold/pdfcraft](https://github.com/storytold/pdfcraft)):

1. อ่านและเขียนภาษาไทยด้วย font ที่มีในเครื่อง ไม่เป็น `[]`
2. เอาการ์ด "Join the ArtCraft community" บนหน้า Home และปุ่ม Discord บนแถบด้านบนออก

**สถานะ (2026-10-31):** ยังไม่เคย compile หรือรันจริง เพราะเครื่องยังไม่มี Rust เมื่อ build และทดสอบผ่านแล้วให้แก้บรรทัดนี้

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

5. build และตรวจตามหัวข้อ "ตรวจหลัง merge"
6. push ขึ้น GitHub:

```bash
git push origin custom/thai-no-ads
```

## จุดที่เราแก้ และวิธีแก้ถ้าโดน conflict

| ไฟล์ | สิ่งที่แก้ | ถ้าโดน conflict |
|---|---|---|
| `crates/fonts/src/craft.rs` | `document_thai_font` ถอยมาใช้ `installed_thai_font` | เก็บการถอยนี้ไว้ และใช้เฉพาะไฟล์ `.ttf` เดี่ยว เพราะ shaping และการฝัง font ไม่รับ `.ttc` |
| `crates/fonts/src/lib.rs` | export `installed_thai_font` | เก็บชื่อนี้ไว้ในรายการ |
| `crates/ui-egui/src/platform/system_fonts.rs` | `thai_fallback`, `thai_candidates`, `thai_face`, `read_face` | เก็บลำดับ font ของเรา ห้ามใส่ Thonburi เพราะสระจะมีวงกลมประติดมา และต้องเลือก face "Text" ของ Sukhumvit Set ไม่ใช่ Thin |
| `crates/ui-egui/src/theme.rs` | `SYSTEM_THAI_FALLBACK` ในรายการ font ของเครื่อง | เก็บไว้ ถ้าต้นทางเพิ่มของเหมือนกันให้เหลือชุดเดียว |
| `crates/edit/src/text_regions/styled.rs` | `thai_face` ใน `FontResolver`, การเลือก font ใน `prepare`, ท้าย `shape_source` | ต้องเก็บทั้งสองจุด: `prepare` ใช้กับตัวอักษรที่ font ใน PDF ไม่มี ส่วน `shape_source` ใช้กับ font ที่ไม่มี GSUB |
| `crates/render/src/system_font.rs`, `crates/render/src/raster.rs` | `Script::Thai` และ `names_thai_family` | เก็บไว้ ใช้กับ PDF ที่ไม่ฝัง font เฉพาะตอนแสดงบนจอ |
| `crates/ui-egui/src/home.rs` | ลบการ์ด community | ลบต่อไป แม้ต้นทางจะแก้ข้อความในการ์ด |
| `crates/ui-egui/src/chrome.rs` | ลบปุ่ม Discord และความกว้างที่กันไว้ให้ปุ่ม | ลบต่อไป `controls_width` ต้องไม่นับปุ่ม Discord |
| `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/app.rs` | ลบ field `community_card_closed` | ลบต่อไป |

test ที่เราแก้ตามไปด้วย: `crates/edit/src/tests.rs` และ `crates/ui-egui/tests/it/` (`content.rs`, `editing.rs`, `fonts.rs`, `links.rs`, `ui.rs`)

ถ้าต้นทาง merge [PR #5132](https://github.com/storytold/pdfcraft/pull/5132) จะมี Thai fallback ซ้ำกับของเรา ให้เหลือชุดเดียว และคงลำดับ font กับ `thai_face` ของเราไว้

ถ้าต้นทางเพิ่มการ์ดหรือปุ่มโปรโมทใหม่ git จะไม่เอาออกให้ ต้องลบเองแล้วเพิ่มลงตารางนี้

## font ที่ใช้

- **ตัวแอป:** Sukhumvit Set, Sathu, Silom, Tahoma ตามลำดับ
- **เขียนลง PDF:** `Sarabun-Regular.ttf`, `THSarabunNew.ttf`, `NotoSansThai-Regular.ttf` ใน `/Library/Fonts` หรือ `~/Library/Fonts` ถ้าไม่มีจึงใช้ Tahoma
- **ถ้า build พร้อม `CRAFT_FONTS_DIR`:** font ไทยจาก craft-fonts จะถูกใช้ก่อน font ในเครื่อง
- `PDFCRAFT_SYSTEM_FONTS=0` ปิดการอ่าน font ในเครื่องทั้งหมด

## ตรวจหลัง merge

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

เปิดแอปด้วย:

```bash
cargo run -p pdfcraft -- ไฟล์.pdf
```

ลองในแอปจริง:

- [ ] ชื่อไฟล์ไทยในแท็บและ Recent ไม่เป็นสี่เหลี่ยม
- [ ] พิมพ์ไทยในช่องค้นหาและช่องข้อความได้
- [ ] คำว่า `ที่ น้ำ ผู้ใช้ กิ่` วรรณยุกต์ไม่ซ้อนสระ
- [ ] Add Text พิมพ์ไทย บันทึก เปิดใหม่ แล้วคัดลอกข้อความได้
- [ ] Edit Text แก้ย่อหน้าใน PDF ไทย บันทึก แล้วเปิดในโปรแกรมอื่นได้
- [ ] PDF ไทยที่ไม่ฝัง font แสดงบนจอได้
- [ ] หน้า Home ไม่มีการ์ด community และแถบด้านบนไม่มีปุ่ม Discord

## ยังไม่รองรับ

- comment และ form field ที่พิมพ์ไทยยังเป็น `?`
- Edit Text กับข้อความที่หมุนอยู่ หรือการเปลี่ยนตัวหนาหรือ font พร้อมกับพิมพ์ไทย จะขึ้น error
- หน้า PDF ที่ไม่ฝัง font แก้เฉพาะตอนดูบนจอ ไม่รวมตอนพิมพ์หรือ export และต้องเป็น font ชนิด CID ที่ชื่อเป็น font ไทย
- ลิงก์ Discord ใน Help menu และหน้า About ยังอยู่ เพราะไม่ใช่โฆษณา
