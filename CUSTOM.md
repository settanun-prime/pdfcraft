# PdfCraft TH: ฉบับปรับแต่งของเรา

Branch `custom/thai-no-ads` เพิ่มสี่เรื่องจากต้นทาง ([storytold/pdfcraft](https://github.com/storytold/pdfcraft)):

1. อ่านและเขียนภาษาไทยด้วย font ที่มีในเครื่อง ไม่เป็น `[]`
2. เอาการ์ด "Join the ArtCraft community" บนหน้า Home และปุ่ม Discord บนแถบด้านบนออก
3. ใช้ไอคอนของเราเอง (คำว่า pdfคับ บนกระดาษสีแดง) เพื่อให้รู้ว่าเป็นเวอร์ชันไทย
4. เอาชื่อและโลโก้ ArtCraft ออกจากแอป เมนู CLI README และโฟลเดอร์ `docs/brand` ตามที่ต้นทางกำหนดไว้สำหรับ fork ที่แก้ไข

**สถานะ (2026-10-31):** build และทดสอบผ่านบน macOS แล้ว ยังไม่เคย build หรือทดสอบบน Windows

## กติกา

- `main` ต้องตรงกับต้นทางเสมอ ห้าม commit งานของเราบน `main`
- งานปรับแต่งทั้งหมดอยู่บน `custom/thai-no-ads`
- ถ้าหน้า GitHub ขึ้นปุ่ม **Discard commits** ห้ามกด
- ใช้แอปที่ build จาก branch นี้เท่านั้น แอปไม่เช็กอัปเดตเอง และปุ่มเช็กอัปเดตแค่เปิดหน้า release ของ fork เพราะ release ของต้นทางไม่มีของที่เราแก้
- ไม่ส่ง branch นี้เป็น PR ไปต้นทาง เพราะเรากำหนดให้ฝัง font ที่ติดตั้งในเครื่อง (แม้เป็น OFL) ลง PDF ซึ่งขัดกับ `AGENTS.md` §1.4 ของต้นทาง และเราเอาแบรนด์ออก

## ใบอนุญาต

นี่คือสรุปของสิ่งที่เราตรวจและทำ ไม่ใช่คำแนะนำทางกฎหมาย

| เรื่อง | กติกา | สิ่งที่เราทำ |
|---|---|---|
| โค้ด | MIT OR Apache-2.0 แก้ได้ แจกต่อได้ ต้องเก็บประกาศลิขสิทธิ์ไว้ | ไม่แตะ `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` รายการสิ่งที่แก้อยู่ในไฟล์นี้ |
| ไอคอน pdfคับ ของเรา | งานต้นฉบับของฉบับนี้ MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) รูปร่างคำลากจาก Kodchasan Bold (SIL OFL 1.1) | สร้างด้วย `packaging/thai/make-logo.py` ไม่มีโลโก้หรือสิงโตของต้นทาง |
| ชื่อและโลโก้ ArtCraft | เป็นเครื่องหมายการค้า ไม่ใช่โอเพนซอร์ส ถ้าเผยแพร่เวอร์ชันที่แก้ไขต้องเอาออก (`docs/brand/LICENSE-brand.txt`, `AGENTS.md`, `README.md`, `NOTICE` ของต้นทางเขียนตรงกัน) | เอาโลโก้ รูปที่มีโลโก้ ปุ่ม เมนู และหน้า About ออกหมด README เป็นของเราเอง |
| คำที่ยังบอกได้ | บอกเป็นข้อความธรรมดาได้ว่า "based on PdfCraft by the ArtCraft team" | หน้า About และ README บอกไว้ และบอกด้วยว่าไม่ใช่ของทางการ ห้ามเขียนว่าทีมนั้นรับรองหรือสนับสนุน |
| font ตอนแสดงบนจอ | อ่าน font ที่ติดตั้งในเครื่องมาวาดบนจอเป็นเรื่องปกติ ไม่คัดลอก ไม่แจก | ใช้ Sukhumvit Set, Sathu, Tahoma แสดงบนจอเท่านั้น |
| font ตอนเขียนลง PDF | ลงไปในไฟล์ที่ส่งต่อได้ จึงต้องมีสัญญาเปิด (`embedding::validate_document_input`) | เขียนด้วย Sarabun หรือ Noto Sans Thai (OFL) เท่านั้น Tahoma และ font ที่มากับ Word ไม่ถูกฝัง แม้ font จะอนุญาตให้ฝังก็ตาม |

**ยังไม่ได้ทำความสะอาด** ชื่อ ArtCraft ที่เป็นข้อความธรรมดายังอยู่ใน `packaging/` (installer ของ Windows, metainfo ของ Linux, flatpak), ไฟล์แปลภาษาใน `crates/ui-egui/src/i18n/`, `AGENTS.md`, `ROADMAP.md` และ `docs/` เราไม่ได้ใช้ไฟล์เหล่านี้ใน `PdfCraft TH.app` ก่อนจะสร้าง installer หรือแจกไฟล์ติดตั้งจาก branch นี้ ต้องแก้ไฟล์ใน `packaging/` ให้ไม่ใช้ชื่อ ArtCraft เป็นผู้เผยแพร่ก่อน ตัว build ฟรีของเรา (`.github/workflows/thai-build.yml`) ไม่ใช้ไฟล์เหล่านั้น: Mac ใช้ `package.sh` กับ `Credits.rtf` ที่เราแก้แล้ว ส่วน Windows ทำแค่ `pdfcraft.exe` ตัวเดียว ไม่ทำ MSI

## เวลาต้นทางมี update

ครั้งแรกบนเครื่องนี้ ตั้งค่าให้ git เก็บ README ของเราไว้เสมอ (คำสั่งนี้เก็บใน `.git/config` ต้องทำใหม่ทุกครั้งที่ clone):

```bash
git config merge.ours.driver true
```

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
python3 packaging/sync-guard.py main
```

ถ้า guard ไม่ผ่าน ห้ามรันบรรทัดถัดไป ให้ไปอ่านหัวข้อ "ถ้าใบอนุญาตของต้นทางเปลี่ยน" ด้านล่าง ถ้าผ่านค่อย merge:

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

merge ผ่านไม่ได้แปลว่างานเรายังอยู่ เพราะต้นทางอาจเพิ่มการ์ดโปรโมทใหม่ เพิ่มโลโก้ หรือเปลี่ยนโค้ดที่เราเรียกใช้โดยไม่เกิด conflict ถ้าสคริปต์ไม่ผ่าน มันจะบอกไฟล์และเรื่องที่ผิด ให้แก้ตามตารางด้านล่างแล้วรันซ้ำ

6. build และตรวจตามหัวข้อ "ตรวจหลัง merge"
7. push ขึ้น GitHub:

```bash
git push origin custom/thai-no-ads
```

## ถ้าใบอนุญาตของต้นทางเปลี่ยน

`packaging/sync-guard.py` เทียบไฟล์ใบอนุญาตของต้นทาง (`LICENSE*`, `COPYING`, `NOTICE` ที่ราก และ `docs/brand/LICENSE-brand.txt`) กับของเราทีละไฟล์ ถ้ามีไฟล์ไหนถูกแก้ เพิ่ม หรือลบ มันรายงานชื่อไฟล์และจบด้วย exit 1 กติกาเมื่อ guard ไม่ผ่าน:

1. **ห้าม merge `main` เข้ามา** ถ้า merge ไปแล้วแต่ยังไม่ commit ให้ `git merge --abort` ถ้า commit หรือ push ไปแล้ว ให้หยุดและถามเจ้าของก่อน อย่าใช้ `reset --hard` เอง
2. อ่านว่าเปลี่ยนอะไร: `git diff HEAD main -- <ไฟล์ที่ guard รายงาน>`
3. ถ้าเปลี่ยนแค่ตัวสะกดหรือปีลิขสิทธิ์ และอ่านแล้วเงื่อนไขเหมือนเดิม ซิงก์ต่อได้ แต่ให้เขียนในข้อความ commit ของการ merge ว่าได้อ่านแล้ว
4. ถ้าเงื่อนไขเข้มขึ้น หรือไม่แน่ใจ ให้ **หยุดซิงก์**: `custom/thai-no-ads` เดินต่อเองจากโค้ดที่ผ่าน guard ครั้งล่าสุด ไม่ดึง `main` อีก และตั้ง tag ที่คอมมิตนั้นไว้ว่าเป็นตัวสุดท้ายที่ใบอนุญาตยังเป็นแบบเดิม (`git tag thai-frozen-YYYYMMDD`) อย่าลบประวัติ
5. หลังหยุด การแก้บั๊กและแพตช์ความปลอดภัยเป็นหน้าที่เราเอง ต้นทางจะไม่ตามมาให้อีก
6. ถ้าภายหลังอยากหยิบของจากต้นทาง ต้องอ่านใบอนุญาตใหม่ก่อน และหยิบได้เฉพาะคอมมิตที่เขียนก่อนวันที่ใบอนุญาตเปลี่ยน (ดูวันที่จาก `git log` ของไฟล์ใบอนุญาต) เรื่องนี้ควรให้ผู้รู้ดูก่อน

ตามที่เข้าใจ ใบอนุญาต MIT และ Apache-2.0 ของโค้ดที่เรารับมาแล้วไม่ถูกเปลี่ยนย้อนหลัง แต่เอกสารนี้ไม่ใช่คำปรึกษากฎหมาย และ guard ตรวจแค่ว่าข้อความในไฟล์ใบอนุญาตเปลี่ยนหรือไม่ ไม่ได้ตัดสินว่าเงื่อนไขใหม่ยอมรับได้หรือไม่ ส่วนนั้นเป็นการตัดสินใจของเจ้าของ fork

## จุดที่เราแก้ และวิธีแก้ถ้าโดน conflict

| ไฟล์ | สิ่งที่แก้ | ถ้าโดน conflict |
|---|---|---|
| `crates/fonts/src/craft.rs` | `document_thai_font` ถอยมาใช้ `installed_thai_font` ซึ่งต้องผ่าน `validate_document_input`, และ `standalone_face` แยก face ออกจาก `.ttc` | เก็บไว้ทั้งหมด ห้ามเอา Tahoma หรือ font ใน Word เข้ารายการที่จะฝังลง PDF เด็ดขาด เลือก face จากชื่อเต็ม "... Regular" เพราะ Thin กับ Medium ก็เรียกตัวเองว่า Regular |
| `crates/fonts/src/lib.rs` | export `installed_thai_font` | เก็บชื่อนี้ไว้ในรายการ |
| `crates/fonts/src/shaped_text.rs` | ข้อความ error ของ `thai_message` บอกให้ติดตั้ง Sarabun หรือ Noto Sans Thai | เก็บคำว่า `CRAFT_FONTS_DIR` และ `install Sarabun` ไว้ เพราะมี test ตรวจ |
| `crates/ui-egui/src/platform/system_fonts.rs` | `thai_fallback`, `thai_candidates`, `thai_face`, `read_face` | เก็บลำดับ font ของเรา ห้ามใส่ Thonburi เพราะสระจะมีวงกลมประติดมา และต้องเลือก face "Text" ของ Sukhumvit Set ไม่ใช่ Thin |
| `crates/ui-egui/src/theme.rs` | `SYSTEM_THAI_FALLBACK` ในรายการ font ของเครื่อง | เก็บไว้ ถ้าต้นทางเพิ่มของเหมือนกันให้เหลือชุดเดียว |
| `crates/fonts/src/plan.rs`, `crates/fonts/src/script.rs` | `document_faces` ใส่ `document_thai_font` ไว้ท้ายรายการ และ `glyph_in_face` ยอมรับสระบนล่างกับวรรณยุกต์ไทยที่กว้างเป็นศูนย์ เพื่อให้ผลลัพธ์ OCR แบบแก้ไขได้เขียนไทยได้ ไม่กลายเป็น `?` | เก็บไว้ |
| `crates/render/src/system_font.rs`, `crates/render/src/raster.rs` | `Script::Thai` และ `names_thai_family` | เก็บไว้ ใช้กับ PDF ที่ไม่ฝัง font เฉพาะตอนแสดงบนจอ ห้ามเอา Tahoma ขึ้นก่อน เพราะไม่มี shaping แล้ววรรณยุกต์จะหาย |
| `crates/ui-egui/src/home.rs` | ลบการ์ด community | ลบต่อไป แม้ต้นทางจะแก้ข้อความในการ์ด |
| `crates/ui-egui/src/chrome.rs` | ลบปุ่ม Discord และความกว้างที่กันไว้ให้ปุ่ม | ลบต่อไป `controls_width` ต้องไม่นับปุ่ม Discord |
| `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/app.rs` | ลบ field `community_card_closed` | ลบต่อไป |
| `crates/ui-egui/src/widgets.rs` | ลบ `artcraft_logo` และ `artcraft_mark` เปลี่ยน `community_links` เป็น `project_links` | ลบต่อไป ถ้าต้นทางเพิ่มฟังก์ชันวาดโลโก้ใหม่ให้ลบด้วย |
| `crates/ui-egui/src/dialogs.rs` | หน้า About ใช้ไอคอน PdfCraft แทนโลโก้ ลบแถว "Part of" และใส่ข้อความว่าไม่ใช่ของทางการ | เก็บไอคอนและข้อความของเรา รับส่วนอื่นของต้นทาง |
| `crates/engine/src/links.rs`, `crates/engine/src/commands.rs`, `crates/ui-egui/src/menus.rs` | ลบรายการ `help.discord`, `help.website` และค่า `DISCORD`, `WEBSITE` | ลบต่อไป ถ้าต้นทางเพิ่มลิงก์ชุมชนใหม่ให้ลบด้วย |
| `apps/pdfcraft-cli/src/main.rs` | ลบบรรทัด Discord ใน help และ `--version` | ลบต่อไป |
| `README.md`, `.gitattributes` | README เป็นของเราเอง และมีบรรทัด `README.md merge=ours` | ไม่ควรโดน conflict ถ้าตั้ง `merge.ours.driver` แล้ว ถ้าโดน ให้เอาของเรา (`git checkout --ours README.md`) |
| `docs/brand/` (ยกเว้น `LICENSE-brand.txt`), `docs/images/pdfcraft-tools.png` | ลบโลโก้และภาพที่มีโลโก้ติดอยู่ แต่เก็บ `LICENSE-brand.txt` ไว้ไฟล์เดียว เพราะด่าน `cargo xtask license-check` ของต้นทางบังคับให้มี (ดู `LICENSES.toml`) | ถ้าต้นทางแก้หรือเพิ่มไฟล์ใน `docs/brand/` ให้รับ `LICENSE-brand.txt` ฉบับใหม่ของต้นทาง ที่เหลือลบต่อด้วย `git rm` |
| `ATTRIBUTION.toml`, `ATTRIBUTION.md` | ลบรายการของ `docs/brand/*` และ `pdfcraft-tools.png` และรายการไอคอนเป็นของเรา (ผู้สร้าง ที่มา และ hash) | รวมรายการใหม่ของต้นทางเข้ามา ห้ามเอารายการ `docs/brand/*` กลับมา แล้วรัน `cargo xtask assets --write` |
| `packaging/macos/Credits.rtf` | ข้อความเป็นของเรา ไม่มีลิงก์ Discord หรือเว็บ ArtCraft และบอกว่าเป็นฉบับไม่เป็นทางการ | เอาของเรา ถ้าต้นทางแก้ไฟล์นี้ให้ดูว่าเพิ่มลิงก์โปรโมทไหม แล้วรับเฉพาะส่วนที่ไม่ใช่โปรโมท |
| `LICENSES.toml` | hash ของ `assets/app-icon/LICENSE.txt` เปลี่ยนเป็นของข้อความใบอนุญาตไอคอนใหม่ของเรา (ด่าน `license-check` บังคับให้ hash ตรง) | ถ้าโดน conflict ให้รับของต้นทางทั้งไฟล์ แล้วแก้บรรทัด `sha256` ของรายการ `assets/app-icon/LICENSE.txt` เป็นค่าจาก `shasum -a 256 assets/app-icon/LICENSE.txt` |
| `crates/ocr/src/pp.rs`, `crates/ocr/src/ppocr.rs`, `crates/ocr/build.rs` | engine PP-OCR (`PpOcrRecognizer`, id `ppocr`) อ่านไทย จีน และละตินมีเครื่องหมาย ใช้ตัวหาบรรทัดของ ocrs กับโมเดลอ่านของ PP-OCR ส่วน `build.rs` ฝังโมเดลลง exe | เป็นไฟล์ของเราเอง ต้นทางเขียน OCR ใหม่ทั้งชุดใน #977 (ocrs กับ Tesseract) แต่ทั้งคู่อ่านไทยไม่ได้: ทดสอบกับเกียรติบัตรจริงเมื่อ 2026-11-10 ocrs อ่านออกมาเป็นตัวมั่ว และ Tesseract ไม่มีภาษาไทยในรายการ |
| `crates/ocr/src/lib.rs`, `crates/engine/src/ocr.rs` | `LANGUAGES` รวมภาษาของ PP-OCR, `recognizers` ส่งภาษาของ PP-OCR ไปที่ `ppocr` (ยกเว้นผู้ใช้เลือก Tesseract), `available_for`, `read_pictures` และ `page_is_picture` | รับโค้ดของต้นทางก่อน แล้วใส่จุดเหล่านี้กลับ `custom-check` จะบอกถ้าหาย |
| `crates/engine/src/ocr/layer.rs` | ชั้นข้อความใช้ฟอนต์ Type 0 ที่มี glyph ว่าง แทน Type 3 ว่างของต้นทาง เพราะ PDFKit ของ Apple มองไม่เห็นข้อความใน Type 3 ว่าง | รับ `validate_words` ของต้นทาง แต่ห้ามเอา Type 3 กลับมา |
| `crates/ui-egui/src/workflows/ocr_ui.rs`, `crates/ui-egui/tests/it/ocr.rs` | หน้าต่าง OCR เปิดเป็นภาษาไทย และทั้งหน้าต่างกับโหมดแก้ไขอ่านรูปบนหน้าที่มีข้อความจริงด้วย (เกียรติบัตรที่พิมพ์ชื่อทับรูป) ส่วนต้นทางข้ามหน้าแบบนี้ | เก็บไว้ test ของต้นทาง `editing_a_page_with_text_over_a_background_picture_changes_nothing` ถูกแทนด้วย test ของเรา |
| `apps/pdfcraft/src/main.rs`, `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/platform/updates.rs` | ไม่เช็กอัปเดตตอนเปิดแอป ปุ่มเช็กอัปเดตเปิดหน้า release ของ fork และ `UPDATE_APP.repo` เป็น fork | เก็บไว้ ถ้าปล่อยให้แอปถามต้นทาง รุ่นใหม่ของเขาจะเขียนทับฉบับไทย |
| `.github/workflows/thai-build.yml`, `packaging/thai/` | ไฟล์ของเราเอง ต้นทางไม่มี จึงไม่ชน workflow ดึง `craft-i18n` (ตาม `craft-i18n.rev`) และ `craft-fonts` (pin เดียวกับ `ci.yml`) มาก่อน build | เก็บไว้ทั้งหมด ถ้าต้นทางเปลี่ยน pin ของ craft-fonts ใน `ci.yml` ให้เปลี่ยนตาม |

ตั้งแต่ 2026-11 คำแปลทุกภาษาอยู่ใน `storytold/craft-i18n` ไม่ได้อยู่ในโปรเจกต์แล้ว ถ้า build โดยไม่ตั้ง `CRAFT_I18N_DIR` หน้าจอจะเป็นภาษาอังกฤษอย่างเดียว ในเครื่องนี้มีสำเนาอยู่ที่ `~/pdfcraft-deps/craft-i18n` และ `~/pdfcraft-deps/craft-fonts` ให้ build และรันเทสด้วย:

```bash
export CRAFT_I18N_DIR=$HOME/pdfcraft-deps/craft-i18n CRAFT_FONTS_DIR=$HOME/pdfcraft-deps/craft-fonts
```

ถ้า pin เปลี่ยน ให้ดึงใหม่ตาม `craft-i18n.rev` และ `craft-fonts-ref` ใน `ci.yml` เครื่องนี้ตั้งภาษาไทย เทส UI ที่ไม่ได้ตั้งภาษาเองจะเห็นป้ายเป็นไทยแล้วล้ม ให้รันด้วย `LC_ALL=en_US.UTF-8`

test ที่เราแก้ตามไปด้วย: `crates/edit/src/tests.rs`, `crates/engine/src/links/tests.rs` และ `crates/ui-egui/tests/it/` (`content.rs`, `editing.rs`, `fonts.rs`, `links.rs`, `ui.rs`)

ถ้าต้นทาง merge [PR #5132](https://github.com/storytold/pdfcraft/pull/5132) จะมี Thai fallback ซ้ำกับของเรา ให้เหลือชุดเดียว และคงลำดับ font กับ `thai_face` ของเราไว้

ถ้าต้นทางเพิ่มการ์ดหรือปุ่มโปรโมทใหม่ git จะไม่เอาออกให้ ต้องลบเองแล้วเพิ่มลงตารางนี้

ถ้าไฟล์ใน `assets/app-icon/` โดน conflict หรือต้นทางเปลี่ยนไอคอน ให้เอาของเรา (ไอคอนเป็นของเราเองแล้ว) จะได้ไม่ต้องตามต้นทาง ถ้าไฟล์ไอคอนเสียหรืออยากวาดใหม่ ให้รันสคริปต์ มันเขียนไฟล์ไอคอนทุกขนาดและอัปเดต hash ใน `ATTRIBUTION.toml` ให้เอง:

```bash
git checkout --ours assets/app-icon
```

```bash
python3 packaging/thai/make-logo.py
```

```bash
cargo xtask assets --write
```

ต้องใช้ Pillow ของ Python และรันบน Mac เพื่อให้ได้ไฟล์ `.icns`

## Build และแจกไฟล์ (ฟรี ไม่ลงชื่อ)

ไม่ใช้ `release.yml` ของต้นทาง เพราะบน branch `release` มันบังคับลงชื่อและ notarize ด้วยบัญชี Apple กับ Azure ที่เราไม่มี และ job ของ Mac ใช้เครื่องขนาดใหญ่ที่เสียเงิน ห้ามสร้าง branch ชื่อ `release` ใน fork นี้ เราใช้ `.github/workflows/thai-build.yml` ซึ่งรันเฉพาะตอน push tag ชื่อ `thai-v<เวอร์ชัน>-<เลข>` (เช่น `git tag thai-v0.16.1-1` แล้ว `git push origin thai-v0.16.1-1`) บนเครื่องมาตรฐานของ GitHub (`macos-latest`, `windows-latest`) ซึ่ง repo สาธารณะใช้ฟรี ใช้ tag เพราะปุ่ม Run workflow ของ GitHub แสดงเฉพาะ workflow ที่อยู่บน branch หลัก (`main`) ซึ่งเราเก็บเป็นสำเนาต้นทางล้วน ๆ ถ้าเปลี่ยน `runs-on` ให้ตรวจก่อนว่าเครื่องนั้นฟรี `custom-check.py` จะแจ้งถ้าไฟล์นี้มีเครื่องขนาดใหญ่

| ระบบ | ได้อะไร | วิธีทำ |
|---|---|---|
| Mac (Apple silicon) | DMG อย่างเดียว (ไม่แจก zip ของ CLI) ลงชื่อแบบ ad hoc ไม่ผ่าน notarize | `packaging/macos/package.sh --arch aarch64` ของต้นทาง |
| Windows x64 | `pdfcraft.exe` ตัวเดียว (ไม่มี CLI) ฝังโมเดล OCR อังกฤษและไทยไว้ข้างใน (`PDFCRAFT_EMBED_MODELS`) กับไฟล์ข้อความรวมใบอนุญาต ไม่มี MSI เพราะไฟล์ WiX ใช้ชื่อ ArtCraft | `packaging/thai/windows-exe.ps1` |

ทุก build ตั้ง `PDFCRAFT_BUILD_LABEL=thai` ให้ `--version` และหน้า About บอกว่าเป็นฉบับนี้ ไฟล์ไม่ได้ลงชื่อ จึงต้องเตือนผู้ใช้ว่า macOS (Gatekeeper) และ Windows (SmartScreen) จะถามยืนยันก่อนเปิด ผลลัพธ์เก็บเป็น artifact 7 วันและ repo เป็นสาธารณะ ใครล็อกอิน GitHub ก็โหลดได้ ก่อนรันให้รัน `python3 packaging/custom-check.py` ให้ผ่านก่อน

**ชื่อเวอร์ชัน:** tag `thai-v<เวอร์ชันต้นทาง>-<เลข>` เช่น `thai-v0.16.1-1` ทำให้ build ติดป้าย `th.1` (ใน `--version`, หน้า About และชื่อไฟล์) และสร้าง **draft release** พร้อมแนบไฟล์ให้ โดยไม่เผยแพร่เอง ต้องเข้าไปอ่านและแก้ข้อความ ตรวจไฟล์ แล้วกด Publish เองบน GitHub ข้อความ release อยู่ใน `packaging/thai/release-notes.md` (ต้องมีประโยคว่าไม่เป็นทางการ) เลขท้ายเพิ่มทีละหนึ่งทุกครั้งที่ปล่อย เราไม่แก้ `version` ใน `Cargo.toml` เพราะจะชนกับต้นทางทุกครั้งที่ซิงก์

### ถ้าเปลี่ยนโลโก้เป็นของเราเอง

ใบอนุญาตแบรนด์ของต้นทางบอกไว้ว่า ฉบับที่แก้ไขต้องเอาเครื่องหมาย ArtCraft ออก "หรือแทนด้วยของตัวเอง" จึงทำได้ ข้อที่ต้องทำ:

1. โลโก้ใหม่ต้องเป็นงานที่เราสร้างเอง หรือมีใบอนุญาตเปิดที่ใช้แก้ไขและแจกต่อได้ ห้ามเลียนเครื่องหมายของคนอื่น และห้ามมีชื่อหรือรูป ArtCraft
2. ใส่รายการใน `ATTRIBUTION.toml` (ผู้สร้าง ใบอนุญาต ไฟล์ใบอนุญาต `sha256`) แล้วรัน `cargo xtask assets --write` มิฉะนั้นด่านตรวจใน CI ล้ม
3. ไอคอนสิงโตเดิมของต้นทางเป็นงานของโครงการเองภายใต้ MIT OR Apache-2.0 เราเปลี่ยนเป็นไอคอนของเราแล้ว (คำว่า pdfคับ บนกระดาษสีแดง วาดโดย `packaging/thai/make-logo.py`) ถ้าจะแก้สีหรือเลย์เอาต์ ให้แก้ค่าในสคริปต์แล้วรันซ้ำ
4. ไฟล์ใน `assets/app-icon/` จะโดน conflict ทุกครั้งที่ต้นทางเปลี่ยนไอคอน ให้เก็บของเรา


## font ที่ใช้

- **ตัวแอป (แสดงบนจอ):** Sukhumvit Set, Sathu, Silom, Tahoma ตามลำดับ
- **เปิดดู PDF ที่ไม่ฝัง font (แสดงบนจอ):** Sukhumvit Set, Sathu, Tahoma ตามลำดับ
- **เขียนลง PDF บน macOS:** ไฟล์ที่ผู้ใช้ติดตั้งเอง (`Sarabun-Regular.ttf`, `THSarabunNew.ttf`, `NotoSansThai-Regular.ttf` ใน `/Library/Fonts` หรือ `~/Library/Fonts`) แล้วจึง `Sarabun.ttc` ที่ macOS โหลดมาไว้ ทุกไฟล์ต้องประกาศสัญญาเปิดในตัว font เอง ถ้าไม่ประกาศจะถูกข้าม
- **เขียนลง PDF บน Windows:** ไฟล์ชื่อเดียวกันใน `C:\Windows\Fonts` หรือ `%LOCALAPPDATA%\Microsoft\Windows\Fonts` เท่านั้น Windows ไม่มี Sarabun มาให้ ต้องติดตั้งเอง ถ้าไม่มี Add Text ภาษาไทยจะขึ้น error บอกให้ติดตั้ง
- **ถ้า build พร้อม `CRAFT_FONTS_DIR`:** font ไทยจาก craft-fonts (Noto Sans Thai, OFL) จะถูกใช้ก่อน font ในเครื่อง
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
cargo test -p pdfcraft-engine -j 2
```

```bash
cargo test -p pdfcraft-ui-egui -j 2
```

```bash
cargo xtask assets
```

```bash
cargo xtask license-check
```

ต้นทางใช้ `license-check` แทน `assets` ใน CI แล้ว (`assets` ยังถูกเรียกอยู่ข้างใน) ถ้า `ATTRIBUTION.md` ค้างหลังลบไฟล์ ให้รัน `cargo xtask assets --write` แล้ว commit ด้วย CI ถึงจะผ่าน

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
- [ ] ไอคอนแอปเป็นโลโก้ pdfคับ สีแดง
- [ ] หน้า About มีไอคอน PdfCraft และข้อความว่าไม่ใช่ของทางการ ไม่มีโลโก้ ArtCraft
- [ ] เมนู Help ไม่มี Discord หรือเว็บไซต์ ArtCraft
- [ ] Add Text พิมพ์ไทย บันทึก เปิดใหม่ แล้วคัดลอกข้อความได้
- [ ] Edit Text แก้ย่อหน้าใน PDF ไทย บันทึก แล้วเปิดในโปรแกรมอื่นได้
- [ ] PDF ไทยที่ไม่ฝัง font แสดงบนจอได้
- [ ] หน้า Home ไม่มีการ์ด community และแถบด้านบนไม่มีปุ่ม Discord

## ยังไม่รองรับ

- Windows: CI build และผ่านเทสแล้ว แต่ยังไม่เคยเปิดบนเครื่อง Windows จริง จึงยังไม่ได้ดูผลการแสดงภาษาไทยและลำดับ font
- comment และ form field ที่พิมพ์ไทยยังเป็น `?`
- ตัวหนาและตัวเอียงตอนเขียนไทยลง PDF: ใช้ face Regular อย่างเดียว
- Edit Text กับข้อความที่หมุนอยู่ หรือการเปลี่ยนตัวหนาหรือ font พร้อมกับพิมพ์ไทย จะขึ้น error
- หน้า PDF ที่ไม่ฝัง font แก้เฉพาะตอนดูบนจอ ไม่รวมตอนพิมพ์หรือ export และต้องเป็น font ชนิด CID ที่ชื่อเป็น font ไทย
- OCR ภาษาไทยทำให้ค้นหาและคัดลอกข้อความได้ แต่ยังแก้ข้อความในภาพไม่ได้ และอ่านผิดได้ ดู `docs/thai-ocr-plan.md`
- ไฟล์ติดตั้ง Linux (nfpm, flatpak) และ MSI ของ Windows ยังใช้ชื่อ ArtCraft ใน `packaging/` เราจึงยังไม่สร้างสองอย่างนี้ ดูหัวข้อ "Build และแจกไฟล์"
