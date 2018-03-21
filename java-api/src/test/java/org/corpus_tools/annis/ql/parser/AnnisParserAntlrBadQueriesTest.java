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

import annis.exceptions.AnnisQLSemanticsException;
import annis.exceptions.AnnisQLSyntaxException;
import java.util.Arrays;
import java.util.LinkedList;
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
public class AnnisParserAntlrBadQueriesTest {

  private final AnnisParserAntlr fixture = new AnnisParserAntlr();

  @Parameterized.Parameters(name = "{0}")
  public static Iterable<? extends Object> data() {
    return Arrays.asList("tok &", "#1 . #2", "/das/ & /Haus/", "/das/ & cat=/NP/ & node & #1 . #2",
        "/das/ & /Haus/ & #1 . #3", "node & ( cat=/NP/ & #1 . #2 | /Haus/ & #1 . #2 )",
        "( node & cat=/NP/ & #1 . #2 | /das/ & /Haus/ & #1 . #2 )", "key! =value", "key!", // no ! in IDs
        "tok & tok & #1 .1,norm #2", "tok & tok & #1 .3,norm,4 #2", "tok & tok & #1 .2, #2", "pos == lemma",
        "pos != lemma",
        // catch invalid reflexivity already when parsing the query
        "tok . tok & #1 _o_ #1", "tok . tok & #2 _o_ #2", "tok . tok & #1 _or_ #1", "tok . tok & #2 _or_ #2",
        "tok . tok & #1 _ol_ #1", "tok . tok & #2 _ol_ #2", "tok . tok & #1 _i_ #1", "tok . tok & #2 _i_ #2",
        "tok . tok & #1 _l_ #1", "tok . tok & #2 _l_ #2", "tok . tok & #1 _r_ #1", "tok . tok & #2 _r_ #2",
        "tok . tok & #1 _o_ #1", "tok . tok & #2 _=_ #2");
  };

  @Parameterized.Parameter(value = 0)
  public String aql;

  @Before
  public void setUp() {
  }

  @Test
  public void testBadQueriesAntLR() {

    try {
      fixture.parse(aql, new LinkedList<>());

      fail("bad query passed as good: " + aql);
    } catch (AnnisQLSyntaxException | AnnisQLSemanticsException ex) {
      // ok
    }

  }
}
