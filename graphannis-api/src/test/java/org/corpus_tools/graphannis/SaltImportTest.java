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
package org.corpus_tools.graphannis;

import com.google.common.io.Files;
import java.io.File;
import org.corpus_tools.salt.SaltFactory;
import org.corpus_tools.salt.common.SDocument;
import org.corpus_tools.salt.samples.SampleGenerator;
import org.junit.After;
import org.junit.AfterClass;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;

import static org.corpus_tools.graphannis.QueryToJSON.aqlToJSON;
import static org.junit.Assert.assertEquals;

/**
 *
 * @author thomas
 */
public class SaltImportTest
{
  private API.CorpusStorage storage;
  
  public SaltImportTest()
  {
  }
  
  @BeforeClass
  public static void setUpClass()
  {
  }
  
  @AfterClass
  public static void tearDownClass()
  {
  }
  
  @Before
  public void setUp()
  {
    File tmpDir = Files.createTempDir();
    
    storage = new API.CorpusStorage(tmpDir.getAbsolutePath());
  }
  
  @After
  public void tearDown()
  {
  }

  /**
   * Test of map method, of class SaltImport.
   */
  @Test
  public void testMapComplexExample()
  {
    System.out.println("map");
    SDocument doc = SaltFactory.createSDocument();
    
    SampleGenerator.createTokens(doc);
    SampleGenerator.createMorphologyAnnotations(doc);
    SampleGenerator.createInformationStructureSpan(doc);
    SampleGenerator.createInformationStructureAnnotations(doc);
    
    API.GraphUpdate result = new SaltImport().map(doc.getDocumentGraph()).finish();
    
    storage.applyUpdate("testCorpus", result);
    
    API.StringVector corpus = new API.StringVector("testCorpus");
    
    assertEquals(13, storage.count(corpus, aqlToJSON("node")));
    
    // test that the token are present and have the correct span values
    assertEquals(11, storage.count(corpus, aqlToJSON("tok")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"Is\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"this\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"example\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"more\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"complicated\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"than\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"it\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"appears\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"to\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"be\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("tok=\"?\"")));
    
    // test that the token annotations have been added
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"VBZ\" _=_ \"Is\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"DT\" _=_ \"this\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"NN\" _=_ \"example\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"RBR\" _=_ \"more\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"JJ\" _=_ \"complicated\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"IN\" _=_ \"than\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"PRP\" _=_ \"it\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"VBZ\" _=_ \"appears\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"TO\" _=_ \"to\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\"VB\" _=_ \"be\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("pos=\".\" _=_ \"?\"")));
    
    // test that the precedence works for the token
    assertEquals(1, storage.count(corpus, 
      aqlToJSON("\"Is\" . \"this\" . \"example\" . \"more\" . \"complicated\" . \"than\" . \"it\" . \"appears\" . "
        + "\"to\" . \"be\" . \"?\"")));
    
    // test that coverage works
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"contrast-focus\" _o_ \"Is\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"this\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"example\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"more\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"complicated\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"than\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"it\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"appears\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"to\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"be\"")));
    assertEquals(1, storage.count(corpus, aqlToJSON("Inf-Struct=\"topic\" _o_ \"?\"")));
  }
  
}
