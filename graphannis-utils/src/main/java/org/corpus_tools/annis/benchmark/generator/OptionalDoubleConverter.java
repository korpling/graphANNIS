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
package org.corpus_tools.annis.benchmark.generator;

import java.util.Optional;
import javafx.util.StringConverter;

/**
 *
 * @author thomas
 */
public class OptionalDoubleConverter extends StringConverter<Optional<Double>>
{

  @Override
  public String toString(Optional<Double> object)
  {
    return object == null || !object.isPresent() ? "" : "" + object.get();
  }

  @Override
  public Optional<Double> fromString(String string)
  {
    Optional<Double> result = Optional.empty();
    if(string != null)
    {
      try
      {
        result = Optional.of(Double.parseDouble(string.trim()));
      }
      catch(NumberFormatException ex)
      {
        
      }
    }
    return result;
  }
  
}
