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
package org.corpus_tools.graphannis.capi;

/**
 *
 * @author thomas
 */
public interface AnnisComponentType {

  public static final int Coverage = 0;

  public static final int InverseCoverage = 1;

  public static final int Dominance = 2;

  public static final int Pointing = 3;

  public static final int Ordering = 4;

  public static final int LeftToken = 5;

  public static final int RightToken = 6;

  public static final int PartOfSubcorpus = 7;
  
}
