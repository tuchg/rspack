use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::Stmt;
use swc_core::ecma::ast::{
  ArrayLit, ArrayPat, AssignExpr, AssignOp, AwaitExpr, BindingIdent, CallExpr, Callee, CondExpr,
  Decl, Expr, ExprOrSpread, ExprStmt, Ident, MemberExpr, MemberProp, ParenExpr, Pat, PatOrExpr,
  VarDecl,
};
use swc_core::ecma::ast::{VarDeclKind, VarDeclarator};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::{
  ast::ModuleItem,
  visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
};

pub fn build_async_module<'a>(promises: Vec<&'a String>) -> impl Fold + 'a {
  as_folder(AsyncModuleVisitor { promises })
}

struct AsyncModuleVisitor<'a> {
  promises: Vec<&'a String>,
}

impl<'a> VisitMut for AsyncModuleVisitor<'a> {
  noop_visit_mut_type!();

  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    let last_import = items
      .iter()
      .enumerate()
      .skip_while(|(_, item)| !matches!(item, ModuleItem::Stmt(Stmt::Decl(Decl::Var(_)))))
      .take_while(|(_, item)| matches!(item, ModuleItem::Stmt(Stmt::Decl(Decl::Var(_)))))
      .map(|(i, _)| i)
      .last()
      .map(|i| i + 1);
    if let Some(index) = last_import {
      let mut args = vec![];
      let mut elems = vec![];

      for el in &self.promises {
        let el = format!("_{el}");
        let arg = make_arg(&el);
        let elem = make_elem(&el);
        args.push(arg);
        elems.push(elem);
      }

      let item = vec![make_var(args), make_stmt(elems)];
      items.splice(index..index, item);
    }
  }
}

fn make_arg(arg: &str) -> Option<ExprOrSpread> {
  Some(ExprOrSpread {
    spread: None,
    expr: Box::new(Expr::Ident(Ident {
      span: DUMMY_SP,
      sym: JsWord::from(arg),
      optional: false,
    })),
  })
}

fn make_elem(elem: &str) -> Option<Pat> {
  Some(Pat::Ident(BindingIdent {
    id: Ident {
      span: DUMMY_SP,
      sym: JsWord::from(elem),
      optional: false,
    },
    type_ann: None,
  }))
}

fn make_var(elems: Vec<Option<ExprOrSpread>>) -> ModuleItem {
  let call_expr = CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(Ident {
      span: DUMMY_SP,
      sym: JsWord::from("__webpack_handle_async_dependencies__"),
      optional: false,
    }))),
    args: vec![ExprOrSpread {
      spread: None,
      expr: Box::from(Expr::Array(ArrayLit {
        span: DUMMY_SP,
        elems,
      })),
    }],
    type_args: None,
  };

  let decl = VarDeclarator {
    span: DUMMY_SP,
    name: Pat::Ident(BindingIdent {
      id: Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__webpack_async_dependencies__"),
        optional: false,
      },
      type_ann: None,
    }),
    init: Some(Box::from(call_expr)),
    definite: false,
  };

  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::from(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![decl],
  }))))
}
fn make_stmt(elems: Vec<Option<Pat>>) -> ModuleItem {
  let assign = AssignExpr {
    span: DUMMY_SP,
    op: AssignOp::Assign,
    left: PatOrExpr::Pat(Box::from(ArrayPat {
      span: DUMMY_SP,
      elems,
      optional: false,
      type_ann: None,
    })),
    right: Box::new(Expr::Cond(CondExpr {
      span: DUMMY_SP,
      test: Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("__webpack_async_dependencies__"),
          optional: false,
        })),
        prop: MemberProp::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("then"),
          optional: false,
        }),
      })),
      cons: Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::from(Expr::Await(AwaitExpr {
          span: DUMMY_SP,
          arg: Box::new(Expr::Ident(Ident {
            span: DUMMY_SP,
            sym: JsWord::from("__webpack_async_dependencies__"),
            optional: false,
          })),
        }))),
        args: vec![],
        type_args: None,
      })),
      alt: Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__webpack_async_dependencies__"),
        optional: false,
      })),
    })),
  };

  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Paren(ParenExpr {
      span: DUMMY_SP,
      expr: Box::from(Expr::Assign(assign)),
    })),
  }))
}
