/*
 * Copyright 2013 SFB 632.
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
import java.util.Collection;
import java.util.LinkedList;
import java.util.List;
import org.junit.After;
import org.junit.AfterClass;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

/**
 * @author Thomas Krause <krauseto@hu-berlin.de>
 */
@RunWith(value = Parameterized.class)
public class AnnisParserAntlrParseTest {

  private final AnnisParserAntlr instance = new AnnisParserAntlr();

  private final String aql;
  private final String expected;

  public AnnisParserAntlrParseTest(String aql, String expected) {
    this.aql = aql;
    this.expected = expected;
  }

  @Parameterized.Parameters
  public static Collection<Object[]> data() {
    Object[][] data = new Object[][] { { "tok", "tok" }, { "/abc/", "/abc/" }, { "tok=/abc/", "tok=/abc/" },
        { " (node & cat=/NP/ & #1 . #2) | (/das/ & tok!=/Haus/ & #3 . #4) ",
            "(node & cat=/NP/ & #1 . #2)\n|\n(/das/ & tok!=/Haus/ & #3 . #4)" },
        { "\"das\" & ( x#\"Haus\" | x#\"Schaf\") & #1 . #x",
            "(\"das\" & x#\"Haus\" & #1 . #x)\n|\n(\"das\" & x#\"Schaf\" & #1 . #x)" },
        { "tok=/abc/ . pos . node", "tok=/abc/ & pos & node & #1 . #2 & #2 . #3" },
        { "tok=/abc/ . right#pos", "tok=/abc/ & right#pos & #1 . #right" },
        { "word & word & (#1 . #2 | #2 . #1)", "(word & word & #1 . #2)\n" + "|\n" + "(word & word & #2 . #1)" } };
    return Arrays.asList(data);
  }

  @BeforeClass
  public static void setUpClass() {
  }

  @AfterClass
  public static void tearDownClass() {
  }

  @Before
  public void setUp() {
  }

  @After
  public void tearDown() {
  }

  /**
   * Test of parse method, of class AnnisParserAntlr.
   */
  @Test
  public void testParse() {
    System.out.println("parse " + aql);
    List<Long> corpusList = new LinkedList<>();
    corpusList.add(1234l);

    QueryData result = instance.parse(aql, corpusList);
    assertNotNull(result);
    assertEquals(expected, result.toAQL());

  }

}