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


import com.sun.jna.NativeLong;

import org.corpus_tools.graphannis.CAPI;

/**
 * An API for managing corpora stored in a common location on the file system.
 *    
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class CorpusStorageManager
{
  private final CAPI.AnnisCorpusStorage instance;

  public CorpusStorageManager(String dbDir)
  {
    this.instance = CAPI.annis_cs_new(dbDir);
  }

  public String[] list()
  {
    CAPI.AnnisVec_AnnisCString orig = CAPI.annis_cs_list(instance);
    String[] copy = new String[(int) CAPI.annis_vec_str_size(orig)];
    for(int i=0; i < copy.length; i++)
    {
      copy[i] = CAPI.annis_vec_str_get(orig, new NativeLong(i));
    }
    
    CAPI.annis_free(orig);

    return copy;
  }

  public long count(String corpusName, String queryAsJSON)
  {
    return CAPI.annis_cs_count(instance, corpusName, queryAsJSON);
  }
  
  public String[] find(String corpusName, String queryAsJSON, long offset, long limit)
  {
    CAPI.AnnisVec_AnnisCString vec = CAPI.annis_cs_find(instance,
      corpusName, queryAsJSON, offset, limit);
    
    String[] result = new String[(int) CAPI.annis_vec_str_size(vec)];
    for(int i=0; i < result.length; i++) {
      result[i] = CAPI.annis_vec_str_get(vec, new NativeLong(i));
    }
    
    CAPI.annis_free(vec);
    
    return result;
  }

  public void applyUpdate(String corpusName, GraphUpdate update)
  {
    CAPI.AnnisError result = CAPI.annis_cs_apply_update(instance, corpusName, update.getInstance());
    
    if(result != null) {
      String msg = CAPI.annis_error_get_msg(result);
      CAPI.annis_free(result);
      
      throw new RuntimeException(msg);
    } else {
        CAPI.annis_free(result);
    }
  }

  @Override
  protected void finalize() throws Throwable
  {
    super.finalize();
    if (instance != null)
    {
      CAPI.annis_free(instance);
    }
  }

}
