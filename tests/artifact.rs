extern crate faerie;
#[macro_use]
extern crate target_lexicon;
#[cfg(test)]
extern crate goblin;

use faerie::*;
use std::str::FromStr;

#[test]
fn duplicate_declarations_are_ok() {

    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());

    obj.declare("str.0", faerie::Decl::DataImport {}).expect(
        "initial declaration",
    );

    obj.declare(
        "str.0",
        faerie::Decl::Data {
            global: false,
            writable: false,
        },
    ).expect("declare should be compatible");

    obj.define("str.0", b"hello world\0".to_vec()).expect(
        "define",
    );

    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.declarations(
        vec![
            ("str.0", faerie::Decl::DataImport),
            (
                "str.0",
                faerie::Decl::Data {
                    global: true,
                    writable: false,
                }
            ),
            ("str.0", faerie::Decl::DataImport),
            ("str.0", faerie::Decl::DataImport),
            (
                "str.0",
                faerie::Decl::Data {
                    global: true,
                    writable: false,
                }
            ),

            ("f", faerie::Decl::FunctionImport),
            ("f", faerie::Decl::Function { global: true }),
            ("f", faerie::Decl::FunctionImport),
            ("f", faerie::Decl::FunctionImport),
            ("f", faerie::Decl::Function { global: true }),
        ].into_iter(),
    ).expect("multiple declarations are ok");
}

#[test]
fn multiple_different_declarations_are_not_ok() {

    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());

    obj.declare("f", faerie::Decl::FunctionImport {}).expect(
        "initial declaration",
    );

    assert!(
        obj.declare(
            "f",
            faerie::Decl::Data {
                global: false,
                writable: false,
            },
        ).is_err()
    );
}

#[test]
fn multiple_different_conflicting_declarations_are_not_ok_and_do_not_overwrite() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    assert!(
        obj.declarations(
            vec![
                ("f", faerie::Decl::FunctionImport),
                ("f", faerie::Decl::Function { global: true }),
                ("f", faerie::Decl::FunctionImport),
                ("f", faerie::Decl::FunctionImport),
                ("f", faerie::Decl::Function { global: false }),
            ].into_iter(),
        ).is_err()
    ); // multiple conflicting declarations are not ok
}

#[test]
fn import_declarations_fill_imports_correctly() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.declarations(
        vec![
            ("f", faerie::Decl::FunctionImport),
            ("f", faerie::Decl::FunctionImport),
            ("d", faerie::Decl::DataImport),
        ].into_iter(),
    ).expect("can declare");
    let imports = obj.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 2);
}

#[test]
fn import_declarations_work_with_redeclarations() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.declarations(
        vec![
            ("f", faerie::Decl::FunctionImport),
            ("d", faerie::Decl::DataImport),
            ("d", faerie::Decl::DataImport),
            ("f", faerie::Decl::Function { global: true }),
            ("f", faerie::Decl::FunctionImport),
        ].into_iter(),
    ).expect("can declare");
    let imports = obj.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 1);
}

#[test]
fn import_helper_adds_declaration_only_once() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.import("f", faerie::ImportKind::Function).expect(
        "can import",
    );
    let imports = obj.imports().collect::<Vec<_>>();
    assert_eq!(imports.len(), 1);
}

#[test]
fn reject_duplicate_definitions() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.declarations(
        vec![
            ("f", faerie::Decl::Function { global: true }),
            ("g", faerie::Decl::Function { global: false }),
        ].into_iter(),
    ).expect("can declare");

    obj.define("g", vec![1, 2, 3, 4]).expect("can define");
    // Reject duplicate definition:
    assert!(obj.define("g", vec![1, 2, 3, 4]).is_err());

    obj.define("f", vec![4, 3, 2, 1]).expect("can define");
    // Reject duplicate definitions:
    assert!(obj.define("g", vec![1, 2, 3, 4]).is_err());
    assert!(obj.define("f", vec![1, 2, 3, 4]).is_err());
}

#[test]
fn undefined_symbols() {
    let mut obj = Artifact::new(triple!("x86_64"), "t.o".into());
    obj.declarations(
        vec![
            ("f", faerie::Decl::Function { global: true }),
            ("g", faerie::Decl::Function { global: false }),
        ].into_iter(),
    ).expect("can declare");
    assert_eq!(
        obj.undefined_symbols(),
        vec![String::from("f"), String::from("g")]
    );

    obj.define("g", vec![1, 2, 3, 4]).expect("can define");
    assert_eq!(obj.undefined_symbols(), vec![String::from("f")]);

    obj.define("f", vec![4, 3, 2, 1]).expect("can define");
    assert!(obj.undefined_symbols().is_empty());
}

#[test]
fn vary_output_formats() {
    use target_lexicon::BinaryFormat;
    use goblin::Object;

    let obj = Artifact::new(triple!("x86_64"), "t.o".into());
    assert!(obj.emit().is_err());

    let elf = obj.emit_as(BinaryFormat::Elf).unwrap();
    match Object::parse(&elf).unwrap() {
         Object::Elf(_) => {}
         _ => panic!("emitted as ELF but didn't parse as ELF"),
    }

    let mach = obj.emit_as(BinaryFormat::Macho).unwrap();
    match Object::parse(&mach).unwrap() {
         Object::Mach(_) => {}
         _ => panic!("emitted as MachO but didn't parse as MachO"),
    }

    /* TODO: Enable when COFF is supported.
    let coff = obj.emit_as(BinaryFormat::Coff).unwrap();
    match Object::parse(&coff).unwrap() {
         Object::PE(_) => {}
         _ => panic!("emitted as COFF but didn't parse as COFF"),
    }
    */
}
