/*
 * Copyright 2016 Thomas Krause.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.corpus_tools.annis.ql.parser;

import java.util.Arrays;
import java.util.LinkedList;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.not;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.junit.Assert.assertThat;
import static org.junit.Assert.fail;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

/**
 *
 * @author thomas
 */
@RunWith(value = Parameterized.class)
public class AnnisParserAntlrGoodQueriesTest {

  private final AnnisParserAntlr fixture = new AnnisParserAntlr();

  @Parameterized.Parameters(name = "{0}")
  public static Iterable<? extends Object> data() {
    return Arrays.asList(
        "cat=\"S\" > s#cat=\"S\" &\n" + "((p1#cat = \"NP\" & p2#cat = \"PP\")\n" + "|\n"
            + "(p1#cat = \"PP\" & p2#cat = \"NP\"))\n" + "& #s >* #p1\n" + "& #p1 > #p2",
        "/das/", "\"Dorf\"", "das=/Haus/", "tok", "node", "/das/ & /Haus/ & #1 . #2",
        "node & pos=\"VVFIN\" & cat=\"S\" & node & #3 >[func=\"OA\"] #1 & #3 >[func=\"SB\"] #4 & #3 > #2 & #1 .* #2 & #2 .* #4",
        "/das/ & ( (a#cat=/NP/ & #1 . #a) | (a#/Haus/ & #1 . #a ))",
        "/das/ & ( (cat=/NP/ & #1 . #2) | (/Haus/ & #1 . #3 ))",
        "( (node & cat=/NP/ & #1 . #2) | (/das/ & /Haus/ & #3 . #4) )", "key != \"value\"", "key!=\"value\"",
        "key !=\"value\"", "key!= \"value\"", "tok != \"value\"", "key!= /value.*/", "tok & tok & #1 .2 #2",
        "tok & tok & #1 .norm #2", "#1 . #2 & #2 . #a & tok & tok & a#tok", "tok & tok & #1 .norm* #2",
        "tok & tok & #1 .norm,1 #2", "tok & tok & #1 .norm,1,3 #2", "Inf-Stat=\"new\" & PP & #1 _o_ #2",
        "Topic=\"ab\" & Inf-Stat=\"new\" & #1 _i_ #2", "( (tok))", "node _ident_ node _ident_ pos=\"NN\"",
        "pos . lemma . pos & #1 == #2", "pos . lemma . pos & #1 != #2", "\"\"", "//", "tok=\"\"", "tok=//", "pos=\"\"",
        "pos=//", "pos!=\"\"", "pos!=//", "tok !=\"\"", "tok!=//", "tok & meta::Titel!=\"Steilpass\"",
        // issue #494
        "ZH1Diff=\"INS\" . tok!=\"\"",
        // corner cases of OR
        "\"das\" | \"die\" | \"der\"", "\"das\" | (\"die\" & pos=\"NN\" & #2 . #3) | \"der\"");
  };

  @Parameterized.Parameter(value = 0)
  public String aql;

  @Before
  public void setUp() {
  }

  @Test
  public void testGoodQueryCanBeParsed() {
    try {
      QueryData result = fixture.parse(aql, new LinkedList<>());
      assertThat(result, is(not(nullValue())));
    } catch (Exception ex) {
      ex.printStackTrace(System.err);
      fail("good query throw exception: " + aql);
    }
  }
}
