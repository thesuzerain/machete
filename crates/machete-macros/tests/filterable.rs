use machete_core::filters::{FilterType, FilterableStruct};
use machete_macros::Filterable;

#[test]
fn test_proc_macro_struct_and_enum() {
    #[derive(Filterable, PartialEq, Clone)]
    struct MyStruct {
        #[filter(default, string)]
        basic_string: String,
        #[filter(string)]
        second_string: String,
        #[filter(number)]
        my_int: i32,
        #[filter(iter(MyEnum))]
        my_enum: MyEnum,
    }

    #[derive(Filterable, PartialEq, Clone)]
    enum MyEnum {
        Variant1,
        Variant2(i32),
        Variant3(String),
    }

    impl MyEnum {
        pub fn iter() -> impl Iterator<Item = MyEnum> {
            vec![
                MyEnum::Variant1,
                MyEnum::Variant2(42),
                MyEnum::Variant3("".to_string()),
            ]
            .into_iter()
        }
    }

    impl ToString for MyEnum {
        fn to_string(&self) -> String {
            match self {
                MyEnum::Variant1 => "Variant1".to_string(),
                MyEnum::Variant2(_) => "Variant2".to_string(),
                MyEnum::Variant3(_) => "Variant3".to_string(),
            }
        }
    }

    assert_eq!(
        MyStruct::create_default_filter().field,
        "basic_string".to_string()
    );

    assert_eq!(
        MyStruct::iter_fields(),
        vec!["basic_string", "second_string", "my_int", "my_enum"]
    );

    assert_eq!(
        MyStruct::iter_filter_types_for_field("basic_string"),
        Some(vec![FilterType::Contains("".to_string())])
    );
    assert_eq!(
        MyStruct::iter_filter_types_for_field("second_string"),
        Some(vec![FilterType::Contains("".to_string())])
    );
    assert_eq!(
        MyStruct::iter_filter_types_for_field("my_int"),
        Some(vec![
            FilterType::GreaterThan(0.0),
            FilterType::LessThan(0.0),
            FilterType::EqualToNumber(0.0)
        ])
    );
    assert_eq!(
        MyStruct::iter_filter_types_for_field("my_enum"),
        Some(vec![FilterType::EqualToChoice("Variant1".to_string())])
    );
    assert_eq!(MyStruct::iter_filter_types_for_field("empty"), None);

    assert_eq!(
        MyStruct::iter_filter_variants_for_field("my_enum"),
        Some(vec![
            "Variant1".to_string(),
            "Variant2".to_string(),
            "Variant3".to_string()
        ])
    );
    assert_eq!(
        MyStruct::iter_filter_variants_for_field("basic_string"),
        None
    );
    assert_eq!(
        MyStruct::iter_filter_variants_for_field("second_string"),
        None
    );
    assert_eq!(MyStruct::iter_filter_variants_for_field("my_int"), None);
    assert_eq!(MyStruct::iter_filter_variants_for_field("empty"), None);
}
