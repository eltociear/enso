package org.enso.interpreter.node.expression.builtin.error;

import org.enso.interpreter.dsl.BuiltinType;
import org.enso.interpreter.node.expression.builtin.Builtin;
import org.enso.interpreter.node.expression.builtin.UniquelyConstructibleBuiltin;

import java.util.List;

@BuiltinType
public class InexhaustivePatternMatchError extends UniquelyConstructibleBuiltin {
  @Override
  protected List<String> getConstructorParamNames() {
    return List.of("scrutinee");
  }
}
