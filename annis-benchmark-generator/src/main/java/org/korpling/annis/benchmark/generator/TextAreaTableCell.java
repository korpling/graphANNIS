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
package org.korpling.annis.benchmark.generator;

import javafx.beans.value.ChangeListener;
import javafx.beans.value.ObservableValue;
import javafx.event.EventHandler;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TextArea;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.text.Text;
import javafx.util.Callback;

/**
 *
 * @author thomas
 */
public class TextAreaTableCell<S> extends TableCell<S, String>
{
  
  private Text textNode;
  private TextArea textArea;

  @Override
  public void startEdit()
  {
    super.startEdit();
    textArea = new TextArea(getItem());
    textArea.setOnKeyPressed(new EventHandler<KeyEvent>()
    {
      @Override
      public void handle(KeyEvent evt)
      {
        if (evt.getCode() == KeyCode.ENTER && evt.isControlDown())
        {
          commitEdit(textArea.getText());
          evt.consume();
          
        }
      }
    });
    setGraphic(textArea);
  }

  @Override
  public void commitEdit(String newValue)
  {
    super.commitEdit(newValue);
    textNode.textProperty().set(newValue);
    setGraphic(textNode);
  }

  @Override
  public void cancelEdit()
  {
    super.cancelEdit();
    setGraphic(textNode);
  }
  
  
  
  
  @Override
  protected void updateItem(String item, boolean empty)
  {
    super.updateItem(item, empty);
    if (isEmpty())
    {
      setGraphic(null);
    }
    else if (isEditing())
    {
      textArea.setText(item);
      textArea.wrapTextProperty().set(true);
      setGraphic(textArea);
    }
    else
    {
      textNode = new Text(item);
      textNode.wrappingWidthProperty().bind(widthProperty());
      setGraphic(textNode);
    }
  }
  
  public static <S> Callback<TableColumn<S, String>, TableCell<S, String>> forTableColumn()
  {
    return (TableColumn<S, String> param) -> new TextAreaTableCell<>();
  }

}
