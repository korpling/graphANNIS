/*
 * Copyright 2018 Thomas Krause.
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
package org.corpus_tools.graphannis.api;

import com.google.common.io.Files;
import java.io.File;
import java.util.Arrays;
import java.util.List;
import org.corpus_tools.graphannis.SaltImport;
import org.corpus_tools.salt.common.SDocument;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.common.SaltProject;
import org.corpus_tools.salt.samples.SampleGenerator;
import org.junit.After;
import org.junit.AfterClass;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;
import static org.junit.Assert.*;

/**
 *
 * @author thomas
 */
public class CorpusStorageManagerTest
{
  private CorpusStorageManager storage;
  
  public CorpusStorageManagerTest()
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
    
    File logfile =  new File(tmpDir, "graphannis.log");
    System.out.println("logging to " + logfile.getAbsolutePath());
    storage = new CorpusStorageManager(tmpDir.getAbsolutePath(), 
    logfile.getAbsolutePath(), LogLevel.Trace);
  }
  
  @After
  public void tearDown()
  {
  }

 

  /**
   * Test of subgraph method, of class CorpusStorageManager.
   
  /**
   * Test of subcorpusGraph method, of class CorpusStorageManager.
   */
  @Test
  public void testSubcorpusGraph()
  {
    System.out.println("subcorpusGraph");
    
    String corpusName = "subcorpusExample";
    
    SaltProject p = SampleGenerator.createSaltProject();

    {
    
      SaltImport i = new SaltImport();
      for(SDocument d : p.getCorpusGraphs().get(0).getDocuments()) {
        i.map(d.getDocumentGraph());
      }
      storage.applyUpdate(corpusName, i.finish());

       
    }
    
    SDocumentGraph docOrig = p.getCorpusGraphs().get(0).getDocuments().get(0).getDocumentGraph();
    
    // repeat several times to make it more likely that garbage collection is run
    for(int i=0; i < 25; i++) {
      System.out.println("subcorpusGraph run " + (i+1));
       SDocumentGraph docCreated = storage.subcorpusGraph(corpusName, Arrays.asList(docOrig.getId()));
       docCreated = null;
       System.gc();
    }
  }

  
  
}
