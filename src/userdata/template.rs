use std::sync::LazyLock;

static DEFAULT_ENV: LazyLock<Env> = LazyLock::new(Env::default);

pub struct Template;

impl mlua::UserData for Template {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("library", |_lua, opts: Option<mlua::Table>| {
            if let Some(opts) = opts {
                Env::new_with_opts(opts)
            } else {
                Ok(Env::default())
            }
        });
        methods.add_function(
            "renderstring",
            |lua, (src, ctx): (String, Option<mlua::Table>)| {
                let ctx = match ctx {
                    Some(tbl) => tbl,
                    None => lua.create_table()?,
                };
                DEFAULT_ENV
                    .render_string(&src, &ctx)
                    .map_err(mlua::Error::external)
            },
        );
    }
}

struct Env {
    env: minijinja::Environment<'static>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        let mut env = minijinja::Environment::new();
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);

        Self { env }
    }

    pub fn new_with_opts(opts: mlua::Table) -> mlua::Result<Self> {
        let mut env = minijinja::Environment::new();
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);

        // Load templates from a directory path
        if let Ok(dir) = opts.get::<std::path::PathBuf>("directory") {
            env.set_loader(minijinja::path_loader(dir));
        }

        // Inline templates
        if let Ok(tbl) = opts.get::<mlua::Table>("templates") {
            for pair in tbl.pairs() {
                let (key, value): (String, String) = pair?;
                env.add_template_owned(key, value)
                    .map_err(mlua::Error::external)?;
            }
        }

        // Autoescape behavior
        if let Ok(autoescape) = opts.get::<String>("autoescape") {
            match autoescape.to_lowercase().as_str() {
                "html" => env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html),
                "json" => env.set_auto_escape_callback(|_| minijinja::AutoEscape::Json),
                "none" => env.set_auto_escape_callback(|_| minijinja::AutoEscape::None),
                _ => return Err(mlua::Error::external("invalid autoescape")),
            }
        }

        if let Ok(keep_trailing_newline) = opts.get::<bool>("keep_trailing_newline") {
            env.set_keep_trailing_newline(keep_trailing_newline);
        }

        if let Ok(trim_blocks) = opts.get::<bool>("trim_blocks") {
            env.set_trim_blocks(trim_blocks);
        }

        if let Ok(lstrip_blocks) = opts.get::<bool>("lstrip_blocks") {
            env.set_lstrip_blocks(lstrip_blocks);
        }

        Ok(Self { env })
    }

    pub fn render_string(&self, src: &str, ctx: &mlua::Table) -> mlua::Result<String> {
        self.env.render_str(src, ctx).map_err(mlua::Error::external)
    }
}

impl mlua::UserData for Env {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "renderstring",
            |lua, this, (src, ctx): (String, Option<mlua::Table>)| {
                let ctx = match ctx {
                    Some(tbl) => tbl,
                    None => lua.create_table()?,
                };
                this.render_string(&src, &ctx)
            },
        );

        methods.add_method(
            "render",
            |lua, this, (name, ctx): (String, Option<mlua::Table>)| {
                let ctx = match ctx {
                    Some(tbl) => tbl,
                    None => lua.create_table()?,
                };

                this.env
                    .get_template(&name)
                    .map_err(mlua::Error::external)?
                    .render(&ctx)
                    .map_err(mlua::Error::external)
            },
        );

        methods.add_method_mut("add", |_lua, this, (name, src): (String, String)| {
            this.env
                .add_template_owned(name, src)
                .map_err(mlua::Error::external)
        });

        methods.add_method_mut("remove", |_lua, this, name: String| {
            this.env.remove_template(&name);
            Ok(())
        });

        methods.add_method_mut("clear", |_lua, this, ()| {
            this.env.clear_templates();
            Ok(())
        });
    }
}
