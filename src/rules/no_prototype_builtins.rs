// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;

use swc_ecmascript::ast::CallExpr;
use swc_ecmascript::ast::Expr;
use swc_ecmascript::ast::ExprOrSuper;
use swc_ecmascript::visit::noop_visit_type;
use swc_ecmascript::visit::Node;
use swc_ecmascript::visit::Visit;

const BANNED_PROPERTIES: &[&str] =
  &["hasOwnProperty", "isPrototypeOf", "propertyIsEnumberable"];

pub struct NoPrototypeBuiltins;

const CODE: &str = "no-prototype-builtins";

fn get_message(prop: &str) -> String {
  format!(
    "Access to Object.prototype.{} is not allowed from target object",
    prop
  )
}

impl LintRule for NoPrototypeBuiltins {
  fn new() -> Box<Self> {
    Box::new(NoPrototypeBuiltins)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    CODE
  }

  fn lint_program(
    &self,
    context: &mut Context,
    program: &swc_ecmascript::ast::Program,
  ) {
    let mut visitor = NoPrototypeBuiltinsVisitor::new(context);
    visitor.visit_program(program, program);
  }
}

struct NoPrototypeBuiltinsVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoPrototypeBuiltinsVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }
}

impl<'c> Visit for NoPrototypeBuiltinsVisitor<'c> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, call_expr: &CallExpr, _parent: &dyn Node) {
    let member_expr = match &call_expr.callee {
      ExprOrSuper::Expr(boxed_expr) => match &**boxed_expr {
        Expr::Member(member_expr) => {
          if member_expr.computed {
            return;
          }
          member_expr
        }
        _ => return,
      },
      ExprOrSuper::Super(_) => return,
    };

    if let Expr::Ident(ident) = &*member_expr.prop {
      let prop_name = ident.sym.as_ref();
      if BANNED_PROPERTIES.contains(&prop_name) {
        self.context.add_diagnostic(
          call_expr.span,
          CODE,
          get_message(prop_name),
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_prototype_builtins_valid() {
    assert_lint_ok! {
      NoPrototypeBuiltins,
      r#"
  Object.prototype.hasOwnProperty.call(foo, "bar");
  Object.prototype.isPrototypeOf.call(foo, "bar");
  Object.prototype.propertyIsEnumberable.call(foo, "bar");
  Object.prototype.hasOwnProperty.apply(foo, ["bar"]);
  Object.prototype.isPrototypeOf.apply(foo, ["bar"]);
  Object.prototype.propertyIsEnumberable.apply(foo, ["bar"]);
  hasOwnProperty(foo, "bar");
  isPrototypeOf(foo, "bar");
  propertyIsEnumberable(foo, "bar");
  ({}.hasOwnProperty.call(foo, "bar"));
  ({}.isPrototypeOf.call(foo, "bar"));
  ({}.propertyIsEnumberable.call(foo, "bar"));
  ({}.hasOwnProperty.apply(foo, ["bar"]));
  ({}.isPrototypeOf.apply(foo, ["bar"]));
  ({}.propertyIsEnumberable.apply(foo, ["bar"]));
      "#,
    };
  }

  #[test]
  fn no_prototype_builtins_invalid() {
    assert_lint_err! {
      NoPrototypeBuiltins,
      "foo.hasOwnProperty('bar');": [{col: 0, message: get_message("hasOwnProperty")}],
      "foo.isPrototypeOf('bar');": [{col: 0, message: get_message("isPrototypeOf")}],
      "foo.propertyIsEnumberable('bar');": [{col: 0, message: get_message("propertyIsEnumberable")}],
      "foo.bar.baz.hasOwnProperty('bar');": [{col: 0, message: get_message("hasOwnProperty")}],
    }
  }
}
