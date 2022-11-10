use rspack_core::DecoratorOptions;
use swc_ecma_transforms::proposals::decorators;
use swc_ecma_visit::Fold;

pub fn decorator(option: &Option<DecoratorOptions>) -> impl Fold {
  let config = match option {
    Some(DecoratorOptions {
      legacy,
      emit_metadata,
      use_define_for_class_fields,
    }) => decorators::Config {
      legacy: *legacy,
      emit_metadata: *emit_metadata,
      use_define_for_class_fields: *use_define_for_class_fields,
    },
    None => Default::default(),
  };
  decorators(config)
}
