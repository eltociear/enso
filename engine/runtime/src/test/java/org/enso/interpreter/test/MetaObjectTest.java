package org.enso.interpreter.test;

import java.io.ByteArrayOutputStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.net.URI;
import java.nio.file.Paths;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;
import org.enso.interpreter.runtime.type.ConstantsGen;
import org.enso.polyglot.RuntimeOptions;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Engine;
import org.graalvm.polyglot.Language;
import org.graalvm.polyglot.Source;
import org.graalvm.polyglot.Value;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;
import org.junit.Before;
import org.junit.Test;

public class MetaObjectTest {
  private Context ctx;

  @Before
  public void prepareCtx() throws Exception {
    Engine eng =
        Engine.newBuilder()
            .allowExperimentalOptions(true)
            .logHandler(new ByteArrayOutputStream())
            .option(
                RuntimeOptions.LANGUAGE_HOME_OVERRIDE,
                Paths.get("../../distribution/component").toFile().getAbsolutePath())
            .build();
    this.ctx = Context.newBuilder().engine(eng).allowIO(true).allowAllAccess(true).build();
    final Map<String, Language> langs = ctx.getEngine().getLanguages();
    assertNotNull("Enso found: " + langs, langs.get("enso"));
  }

  @Test
  public void checkingAtomMetaObject() throws Exception {
    final URI uri = new URI("memory://callback.enso");
    final Source src = Source.newBuilder("enso", """
    type Atm
        Data x
        End

    data = Atm.Data 5
    end = Atm.End
    """, "atom_test.enso")
            .uri(uri)
            .buildLiteral();

    var module = ctx.eval(src);

    var data = module.invokeMember("eval_expression", "data");
    assertFalse("Non-null result", data.isNull());
    var meta = data.getMetaObject();
    assertNotNull("Non-null meta for " + data, meta);
    assertFalse("Non-null type", meta.isNull());
    assertTrue("Is meta", meta.isMetaObject());
    assertEquals("Atm", meta.getMetaSimpleName());
    assertEquals("atom_test.Atm", meta.getMetaQualifiedName());
    assertFalse("Isn't instance of itself", meta.isMetaInstance(meta));
    assertTrue("Is instance", meta.isMetaInstance(data));
    assertTrue("Eigen-type", meta.getMetaObject().isMetaObject());
    assertEquals("Eigen-type is equal", meta.getMetaObject(), meta.getMetaObject().getMetaObject());

    var end = module.invokeMember("eval_expression", "end");
    assertEquals("Same meta", end.getMetaObject(), data.getMetaObject());
  }

  @Test
  public void checkAllConstantGenValuesArePresent() throws Exception {
    var g = ValuesGenerator.create(ctx);
    var expecting = new HashSet<String>();
    for (var f : ConstantsGen.class.getFields()) {
      var s = (String) f.get(null);
      expecting.add(s);
    }
    var w = new StringBuilder();
    var f = new StringWriter();
    var err = new PrintWriter(f);
    for (var t : g.allTypes()) {
      try {
        var n = t.getMetaQualifiedName();
        assertNotNull("Type " + t + " has meta name", n);
        if (!expecting.remove(n)) {
          w.append("\nNo type removed for ").append(n);
        }
      } catch (AssertionError ex) {
        err.println(ex.getMessage());
      }
    }

    if (f.toString().length() > 0) {
      fail(f.toString());
    }

    if (!expecting.isEmpty()) {
      fail("Should be empty: " + expecting + w);
    }
  }

  @Test
  public void checkAllTypesHaveSomeValues() throws Exception {
    checkAllTypesSatisfy(MetaObjectTest::checkValue);
  }

  @Test
  public void checkAllTypesHaveInstanceOfValues() throws Exception {
    checkAllTypesSatisfy(MetaObjectTest::checkIsInstance);
  }

  @Test
  public void warningIsTransparent() {
    ValuesGenerator g = ValuesGenerator.create(ctx, ValuesGenerator.Language.ENSO);
    for (var v : g.warnings()) {
      assertTrue("Warning is string: " + v, v.isString());
      assertEquals("value", v.asString());
      assertEquals("Warning " + v + " has type Text", g.typeText(), v.getMetaObject());
    }
  }

  @Test
  public void checkArraysAreArrays() {
    var g = ValuesGenerator.create(ctx, ValuesGenerator.Language.ENSO);
    for (var v : g.arrayLike()) {
      var isVector = v.getMetaObject().equals(g.typeVector());
      var isArray = v.getMetaObject().equals(g.typeArray());
      assertTrue("Value " + v + " of type " + v.getMetaObject() + " should either be array or vector (" + isVector + ")", isArray ^ isVector);
    }
  }

  private void checkAllTypesSatisfy(Check check) throws Exception {
    var g = ValuesGenerator.create(ctx);
    var expecting = new LinkedHashSet<Value>();
    for (var t : g.allTypes()) {
      switch (t.getMetaSimpleName()) {
        // represented as primitive values without meta object
        case "Decimal" -> {}
        // has no instances
        case "Array_Proxy" -> {}
        // Warning is transparent and invisible
        case "Warning" -> {}
        default -> expecting.add(t);
      }
    }
    var successfullyRemoved = new HashSet<Value>();
    var w = new StringBuilder();
    for (var v : g.allValues()) {
      check.check(v, null, expecting, successfullyRemoved, w);
    }
    if (!expecting.isEmpty()) {
      fail("These types don't have any values: " + expecting + w);
    }
  }

  @FunctionalInterface
  interface Check {
    void check(Value v, Value type, Set<Value> expecting, Set<Value> successfullyRemoved, StringBuilder w);
  }

  private static void checkValue(Value v, Value type, Set<Value> expecting, Set<Value> successfullyRemoved, StringBuilder w) {
    var t = type == null ? v.getMetaObject() : type;
    if (t == null) {
      return;
    }
    if (!expecting.remove(t)) {
      if (!successfullyRemoved.contains(t)) {
        w.append("\nCannot remove type ").append(t).append(" for value ").append(v);
      }
    } else {
      successfullyRemoved.add(t);
    }
    if (t.hasMetaParents() && t.getMetaParents() instanceof Value p && p.hasArrayElements()) {
      for (long i = 0; i < p.getArraySize(); i++) {
        checkValue(v, p.getArrayElement(i), expecting, successfullyRemoved, w);
      }
    }
  }

  private static void checkIsInstance(Value v, Value nullT, Set<Value> expecting, Set<Value> successfullyRemoved, StringBuilder w) {
    for (var type : new LinkedHashSet<>(expecting)) {
      if (!type.isMetaInstance(v)) {
        continue;
      }
      if (!expecting.remove(type)) {
        if (!successfullyRemoved.contains(type)) {
          w.append("\nCannot remove type ").append(type).append(" for value ").append(v);
        }
      } else {
        successfullyRemoved.add(type);
      }
    }
  }
}