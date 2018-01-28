/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

package org.corpus_tools.graphannis;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.PointerType;


public interface CAPI extends Library {
  
    CAPI INSTANCE = (CAPI) 
            Native.loadLibrary("graphannis_capi",
                               CAPI.class);
    
    
     public static class annis_CorpusStorage extends PointerType {

     }

    
     public annis_CorpusStorage annis_cs_new(String db_dir);
     public void annis_cs_free(annis_CorpusStorage cs); 
     
     public long annis_cs_count(annis_CorpusStorage cs, String corpusName, String queryAsJSON);
     


}
