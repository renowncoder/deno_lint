export default class LintRule extends Visitor {
  static ruleCode() {
    return "no-unreachable-return-statement";
  }

  visitReturnStatement(stmt) {
    /** @type {(boolean|null)} */
    const reachable = ControlFlow.isReachable(stmt);

    if (reachable === false) {
      this.addDiagnostic({
        span: stmt.span,
        message: "unreachable return statement detected",
      });
    }

    return stmt;
  }
}
