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

import com.sun.jna.Structure;
import java.util.Arrays;
import java.util.List;

/**
 *
 * @author thomas
 */
public class AnnisAnnotation extends Structure {
  
  public AnnisAnnoKey key;

  public StringID value;

  @Override
  protected List<String> getFieldOrder()
  {
    return Arrays.asList("key", "value");
  }

  public static class ByReference extends AnnisAnnotation implements Structure.ByReference
  {
  }

  public static class ByValue extends AnnisAnnotation implements Structure.ByValue
  {
  }
  
}
