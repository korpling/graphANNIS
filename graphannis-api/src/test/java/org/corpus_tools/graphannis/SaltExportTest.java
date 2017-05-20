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
import java.util.Set;
import org.corpus_tools.salt.SaltFactory;
import org.corpus_tools.salt.common.SDocument;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.samples.SampleGenerator;
import org.junit.After;
import org.junit.AfterClass;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;

import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.util.DiffOptions;
import org.corpus_tools.salt.util.Difference;
import org.corpus_tools.salt.util.SaltUtil;
import static org.junit.Assert.*;

/**
 *
 * @author thomas
 */
public class SaltExportTest
{
  private API.CorpusStorageManager storage;
  
  public SaltExportTest()
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
    
    storage = new API.CorpusStorageManager(tmpDir.getAbsolutePath());
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
    SDocument doc = SaltFactory.createSDocument();
    
    SampleGenerator.createTokens(doc);
    SampleGenerator.createMorphologyAnnotations(doc);
    SampleGenerator.createInformationStructureSpan(doc);
    SampleGenerator.createInformationStructureAnnotations(doc);
    SampleGenerator.createSyntaxStructure(doc);
    SampleGenerator.createSyntaxAnnotations(doc);
    SampleGenerator.createAnaphoricAnnotations(doc);
    SampleGenerator.createDependencies(doc);
    
    API.GraphUpdate result = new SaltImport().map(doc.getDocumentGraph()).finish();
    
    storage.applyUpdate("testCorpus", result);
    
    SToken sampleTok = doc.getDocumentGraph().getTokens().get(2);
    
    // get a subgraph for the complete document
    API.NodeVector nodeVector = storage.subgraph("testCorpus", new API.StringVector(sampleTok.getId()), 100, 100);
    
    assertTrue(nodeVector.size() > 0);
    
    SDocumentGraph exportedGraph = SaltExport.map(nodeVector);
      
    assertEquals(doc.getDocumentGraph().getNodes().size(), exportedGraph.getNodes().size());
    assertEquals(doc.getDocumentGraph().getTokens().size(), exportedGraph.getTokens().size());

    // TODO: actual diff
  }
  
  
  
}
