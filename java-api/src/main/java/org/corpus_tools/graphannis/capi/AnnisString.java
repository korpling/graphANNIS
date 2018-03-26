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

import com.sun.jna.Pointer;
import com.sun.jna.PointerType;

/**
 *
 * @author thomas
 */
public class AnnisString extends PointerType implements CharSequence {
  
  public synchronized void dispose()
  {
    try {
      if (this.getPointer() != Pointer.NULL) {
        CAPI.annis_str_free(this.getPointer());
      }
    } finally {
      this.setPointer(Pointer.NULL);
    }
  }

  @Override
  protected void finalize() throws Throwable
  {
    this.dispose();
    super.finalize();
  }

  @Override
  public String toString()
  {
    if (getPointer() == Pointer.NULL)
    {
      return "";
    }
    else
    {
      return getPointer().getString(0);
    }
  }

  @Override
  public CharSequence subSequence(int start, int end)
  {
    return toString().subSequence(start, end);
  }

  @Override
  public int length()
  {
    return toString().length();
  }

  @Override
  public char charAt(int index)
  {
    return toString().charAt(index);
  }
  
}
