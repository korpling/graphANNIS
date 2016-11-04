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

import org.corpus_tools.salt.common.SDocumentGraph;

/**
 * A class which helps to import salt documents into graphANNIS.
 * 
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltImport
{
  public static API.GraphUpdate map(SDocumentGraph g)
  {
    API.GraphUpdate update = new API.GraphUpdate();
    
    
    update.finish();
    
    return update;
  }
}
