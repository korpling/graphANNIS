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

import org.corpus_tools.graphannis.api.CorpusStorageManager;

/**
 *
 * @author thomas
 */
public class Main {

  /**
   * @param args the command line arguments
   */
  public static void main(String[] args) {
    if (args.length > 0) {
      CorpusStorageManager manager = new CorpusStorageManager(args[0]);
      if (args.length > 1) {
        String aql = args.length > 2 ? args[2] : "tok";

        System.out.println(manager.count(args[1], QueryToJSON.aqlToJSON(aql)));

      }
    } else {
      System.err.println("You have to a give a database directory as argument.");
    }
  }

}
