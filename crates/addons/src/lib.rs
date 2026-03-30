use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf, absolute},
};

use meralus_storage::ResourceStorage;
use mollie::{
    AdtBuilder, GcPtr, VTableBuilder,
    compiler::{Compiler, cranelift::module::ModuleResult, error::CompileError},
    index::Idx,
    typed_ast::{FileModuleLoader, TypedASTContext},
    typing::{AdtRef, Func, ModuleId, Type, TypeContext, TypeRef},
};
use serde::Deserialize;
use tracing::info;

// type Program = fn(*mut RenderContext, *const GameMetadata);

fn add_builtins(context: &mut TypedASTContext) -> TypeRef {
    let color_ty = AdtBuilder::new_struct(&mut context.type_context, "Color")
        .field::<u8>("red")
        .field::<u8>("green")
        .field::<u8>("blue")
        .finish();

    let color_ty = context.type_context.inst_adt(color_ty, &[]);
    let draw_ctx_ty = AdtBuilder::new_struct(&mut context.type_context, "DrawContext").finish();
    let draw_ctx_ty = context.type_context.inst_adt(draw_ctx_ty, &[]);

    let println_str = context.type_context.types.get_or_add(Type::Func(
        Box::new([context.type_context.core_types.string]),
        context.type_context.core_types.void,
    ));

    context.type_context.register_func_in_module(ModuleId::ZERO, Func {
        postfix: false,
        name: "println_str".to_owned(),
        arg_names: Vec::new(),
        ty: println_str,
    });

    let f32 = context.type_context.core_types.f32;
    let void = context.type_context.core_types.void;

    VTableBuilder::new(context, draw_ctx_ty)
        .func("draw_rect", "DrawContext_draw_rect", [draw_ctx_ty, f32, f32, f32, f32, color_ty], void)
        .finish();

    // func_compiler.compiler.var_ty(
    //     "println",
    //     TypeVariant::function([TypeVariant::one_of([TypeVariant::int64(),
    // TypeVariant::usize()])], TypeVariant::void()), );

    // func_compiler
    //     .compiler
    //     .var_ty("println_str", TypeVariant::function([TypeVariant::string()],
    // TypeVariant::void())); func_compiler
    //     .compiler
    //     .var_ty("println_bool",
    // TypeVariant::function([TypeVariant::boolean()], TypeVariant::void()));
    // func_compiler
    //     .compiler
    //     .var_ty("println_addr", TypeVariant::function([TypeVariant::any()],
    // TypeVariant::void())); func_compiler
    //     .compiler
    //     .var_ty("get_type_idx", TypeVariant::function([TypeVariant::any()],
    // TypeVariant::usize())); func_compiler
    //     .compiler
    //     .var_ty("get_size", TypeVariant::function([TypeVariant::any()],
    // TypeVariant::usize()));

    // let draw_ctx_ty = TypeVariant::structure::<String, Type, _>([]);
    // let metadata_ty =
    // TypeVariant::structure_ir(func_compiler.compiler.jit.module.isa(), [
    //     ("window_width", TypeVariant::float()),
    //     ("window_height", TypeVariant::float()),
    // ]);

    // let color_ty =
    // TypeVariant::structure_ir(func_compiler.compiler.jit.module.isa(), [
    //     ("red", TypeVariant::uint8()),
    //     ("green", TypeVariant::uint8()),
    //     ("blue", TypeVariant::uint8()),
    // ]);

    // let object_fit_ty = TypeVariant::enumeration(["Stretch", "Cover"]);

    // let corner_radius_ty =
    // TypeVariant::structure_ir(func_compiler.compiler.jit.module.isa(), [
    //     ("top_left", TypeVariant::float()),
    //     ("top_right", TypeVariant::float()),
    //     ("bottom_left", TypeVariant::float()),
    //     ("bottom_right", TypeVariant::float()),
    // ]);

    // let draw_ctx_type_idx = func_compiler.compiler.add_type("DrawContext",
    // draw_ctx_ty.clone());

    // func_compiler.compiler.add_type("Color", color_ty.clone());
    // func_compiler.compiler.add_type("CornerRadius",
    // corner_radius_ty.clone()); func_compiler.compiler.add_type("
    // ObjectFit", object_fit_ty.clone());

    // let metadata_ty_idx = func_compiler.compiler.add_type("GameMetadata",
    // metadata_ty.clone());

    // func_compiler.compiler.var("metadata", metadata_ty_idx);
    // func_compiler.compiler.var("context", draw_ctx_type_idx);

    // let draw_rect = func_compiler
    //     .add_native_fn(
    //         "DrawContext_draw_rect",
    //         Some(draw_ctx_ty.clone()),
    //         [
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             color_ty.clone(),
    //         ],
    //         TypeVariant::void(),
    //     )
    //     .unwrap_or_else(|e| panic!("failed to add DrawContext_draw_rect:
    // {e}"));

    // let draw_round_rect = func_compiler
    //     .add_native_fn(
    //         "DrawContext_draw_rrect",
    //         Some(draw_ctx_ty.clone()),
    //         [
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             corner_radius_ty.clone(),
    //             color_ty.clone(),
    //         ],
    //         TypeVariant::void(),
    //     )
    //     .unwrap_or_else(|e| panic!("failed to add DrawContext_draw_rrect:
    // {e}"));

    // let draw_image = func_compiler
    //     .add_native_fn(
    //         "DrawContext_draw_image",
    //         Some(draw_ctx_ty.clone()),
    //         [
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::string(),
    //             object_fit_ty,
    //         ],
    //         TypeVariant::void(),
    //     )
    //     .unwrap_or_else(|e| panic!("failed to add DrawContext_draw_image:
    // {e}"));

    // let draw_round_image = func_compiler
    //     .add_native_fn(
    //         "DrawContext_draw_round_image",
    //         Some(draw_ctx_ty.clone()),
    //         [
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             corner_radius_ty,
    //             TypeVariant::string(),
    //         ],
    //         TypeVariant::void(),
    //     )
    //     .unwrap_or_else(|e| panic!("failed to add
    // DrawContext_draw_round_image: {e}"));

    // let draw_text = func_compiler
    //     .add_native_fn(
    //         "DrawContext_draw_text",
    //         Some(draw_ctx_ty),
    //         [
    //             TypeVariant::float(),
    //             TypeVariant::float(),
    //             TypeVariant::string(),
    //             TypeVariant::string(),
    //             TypeVariant::float(),
    //             color_ty,
    //         ],
    //         TypeVariant::void(),
    //     )
    //     .unwrap_or_else(|e| panic!("failed to add DrawContext_draw_text:
    // {e}"));

    // func_compiler
    //     .create_fallback_vtable(draw_ctx_type_idx, [
    //         ("draw_rect", draw_rect),
    //         ("draw_rrect", draw_round_rect),
    //         ("draw_image", draw_image),
    //         ("draw_round_image", draw_round_image),
    //         ("draw_text", draw_text),
    //     ])
    //     .unwrap_or_else(|e| panic!("failed to create fallback vtable for
    // DrawContext: {e}"));
    draw_ctx_ty
}

#[derive(Debug, Deserialize)]
pub struct AddonInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct AddonPackage {
    #[serde(rename = "addon")]
    pub info: AddonInfo,
}

#[derive(Debug)]
pub struct Addon {
    pub base: PathBuf,
    pub package: AddonPackage,
    pub main: String,
}

impl Addon {
    pub fn load_all<P: AsRef<Path>>(folder: P) -> Vec<Self> {
        fs::read_dir(folder)
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| {
                        let base = entry.path();

                        if base.is_dir() {
                            let package = fs::read(base.join("package.toml")).ok()?;
                            let package = toml::from_slice(&package).ok()?;
                            let main = fs::read_to_string(base.join("src/main.mol")).ok()?;

                            Some(Self { base, package, main })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

struct DataContext<'a> {
    current_mapping: &'a str,
    storage: &'a mut meralus_storage::ResourceStorage,
}

#[derive(Debug, Clone, Copy)]
struct BlockData {
    id: &'static str,
    cull_if_same: bool,
    blocks_light: bool,
    consume_light_level: u8,
    light_level: u8,
    droppable: bool,
    collidable: bool,
    selectable: bool,
}

impl meralus_storage::Block for BlockData {
    fn id(&self) -> &'static str {
        self.id
    }

    fn cull_if_same(&self) -> bool {
        self.cull_if_same
    }

    fn blocks_light(&self) -> bool {
        self.blocks_light
    }

    fn consume_light_level(&self) -> u8 {
        self.consume_light_level
    }

    fn light_level(&self) -> u8 {
        self.light_level
    }

    fn droppable(&self) -> bool {
        self.droppable
    }

    fn collidable(&self) -> bool {
        self.collidable
    }

    fn selectable(&self) -> bool {
        self.selectable
    }
}

impl DataContext<'_> {
    fn register_block(&mut self, data: GcPtr<BlockData>) {
        // println!("[{}] Registering {}", "INFO/BlockLoader  ".bright_green(),
        // data.id.bright_blue().bold());

        self.storage.register_block(self.current_mapping, *data);
    }
}

impl BlockData {
    fn register(context: &mut TypeContext) -> AdtRef {
        AdtBuilder::new_struct(context, "Block")
            .field::<&str>("id")
            .field_default::<bool>("cull_if_same", false)
            .field_default::<bool>("blocks_light", true)
            .field_default::<u8>("consume_light_level", 0)
            .field_default::<u8>("light_level", 0)
            .field_default::<bool>("droppable", true)
            .field_default::<bool>("collidable", true)
            .field_default::<bool>("selectable", true)
            .finish()
    }
}

impl DataContext<'_> {
    fn register(context: &mut TypeContext) -> AdtRef {
        AdtBuilder::new_struct(context, "DataContext").non_gc_collectable().finish()
    }
}

// let world_data_ty = func_compiler
// .checker
// .register_adt(AdtBuilder::new_struct("WorldData").non_gc_collectable().
// finish());
//
// let event_mgr_ty = func_compiler
// .checker
// .register_adt(AdtBuilder::new_struct("EventManager").non_gc_collectable().
// finish());
//
// let (event_mgr_info, _) = func_compiler.checker.instantiate_adt(event_mgr_ty,
// &[]);
//
// let (world_data_info, _) =
// func_compiler.checker.instantiate_adt(world_data_ty, &[]);
//
// let (block_ty_info, _) = func_compiler.checker.instantiate_adt(block_ty,
// &[]);
// let println_str_info = func_compiler.checker.solver.add_info(
// TypeInfo::Func(
// Box::new([FuncArg::Regular(func_compiler.checker.core_types.string)]),
// func_compiler.checker.core_types.void,
// ),
// None,
// );
//
// func_compiler.checker.solver.add_var("data", data_ctx_info);
// func_compiler.checker.solver.add_var("events", event_mgr_info);
// func_compiler.checker.solver.add_var("println_str", println_str_info);
//

// let tick_func_info = func_compiler
// .checker
// .solver
// .add_info(TypeInfo::Func(Box::new([]),
// func_compiler.checker.core_types.void), None);
//
// let world_start_func_info = func_compiler.checker.solver.add_info(
// TypeInfo::Func(Box::new([FuncArg::Regular(world_data_info)]),
// func_compiler.checker.core_types.void), None,
// );
//
// VTableBuilder::new(FieldType::Adt(world_data_ty, AdtKind::Struct,
// Box::new([]))) .func(
// "send_chat_message",
// "WorldData_send_chat_message",
// [world_data_info, func_compiler.checker.core_types.string],
// func_compiler.checker.core_types.void,
// )
// .finish(&mut func_compiler.checker);
//
// VTableBuilder::new(FieldType::Adt(event_mgr_ty, AdtKind::Struct,
// Box::new([]))) .func(
// "on_tick",
// "EventManager_on_tick",
// [event_mgr_info, tick_func_info],
// func_compiler.checker.core_types.void,
// )
// .func(
// "on_world_start",
// "EventManager_on_world_start",
// [event_mgr_info, world_start_func_info],
// func_compiler.checker.core_types.void,
// )
// .finish(&mut func_compiler.checker);
pub type Mappings = HashMap<String, PathBuf>;

pub struct AddonManager {
    addons: Vec<Addon>,
    compiler: Compiler<FileModuleLoader>,
}

impl AddonManager {
    pub fn new<P: AsRef<Path>>(folder: P) -> ModuleResult<Self> {
        Ok(Self {
            addons: Addon::load_all(folder),
            compiler: Compiler::with_symbols(
                FileModuleLoader {
                    current_dir: PathBuf::from("/"),
                },
                [("DataContext_register_block", DataContext::register_block as *const u8)],
            )?,
        })
    }

    pub fn insert_mappings(&self, storage: &mut ResourceStorage) -> io::Result<()> {
        for addon in &self.addons {
            storage
                .mappings
                .insert(addon.package.info.name.clone(), absolute(addon.base.join("resources"))?);
        }

        Ok(())
    }

    pub fn execute(&mut self, storage: &mut ResourceStorage) -> ModuleResult<()> {
        let mut func_compiler = self.compiler.start_compiling();
        let ptr_type = func_compiler.compiler.ptr_type();
        let mut context = DataContext {
            current_mapping: "game",
            storage,
        };

        let block_ty = BlockData::register(&mut func_compiler.type_context.type_context);
        let block_ty = func_compiler.type_context.type_context.inst_adt(block_ty, &[]);
        let data_ctx_ty = DataContext::register(&mut func_compiler.type_context.type_context);
        let data_ctx_ty = func_compiler.type_context.type_context.inst_adt(data_ctx_ty, &[]);
        let void = func_compiler.type_context.type_context.core_types.void;

        VTableBuilder::new(&mut func_compiler.type_context, data_ctx_ty)
            .func("register_block", "DataContext_register_block", [data_ctx_ty, block_ty], void)
            .finish();

        for addon in &self.addons {
            info!(target: "addons", "Loading {} v{}", addon.package.info.name, addon.package.info.version);

            context.current_mapping = &addon.package.info.name;
            func_compiler.module_loader.current_dir = addon.base.join("src");

            let name = format!("{}_<main>", addon.package.info.name);

            if let Err(CompileError::Type(errors)) = func_compiler.compile(name.as_str(), [("data", ptr_type, data_ctx_ty)], None, &addon.main) {
                let file = addon.base.join("src/main.mol").display().to_string();
                let file = file.as_str();

                for error in errors {
                    let span = (file, error.span.start..error.span.end);
                    let mut report = ariadne::Report::build(ariadne::ReportKind::Error, span.clone()).with_config(ariadne::Config::new().with_compact(true));

                    error.value.add_to_report(span, &mut report, &func_compiler.type_context.type_context);

                    report.finish().print((file, ariadne::Source::from(&addon.main))).unwrap();
                }
            }

            unsafe { func_compiler.compiler.get_func::<fn(&mut DataContext)>(name).unwrap()(&mut context) };
        }

        Ok(())
    }
}
