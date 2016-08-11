/*
 * Copyright 2016 Thomas Krause <thomaskrause@posteo.de>.
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

import java.net.URL;
import java.util.Arrays;
import java.util.ResourceBundle;
import java.util.concurrent.ExecutionException;

import org.korpling.graphannis.QueryToJSON;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;
import annis.ql.parser.SemanticValidator;
import javafx.application.Platform;
import javafx.concurrent.Task;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.TextArea;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;

/**
 * FXML Controller class
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class QueryConverterController implements Initializable
{
  
  private final static Logger log = LoggerFactory.getLogger(QueryConverterController.class);

  @FXML
  private TextArea aqlInput;

  @FXML
  private TextArea jsonOutput;

  @FXML
  private Button convertButton;
  
  private AnnisParserAntlr parser;
  
  /**
   * Initializes the controller class.
   */
  @Override
  public void initialize(URL url, ResourceBundle rb)
  {
    parser = new AnnisParserAntlr();
    parser.setPrecedenceBound(50);
    parser.setPostProcessors(Arrays.asList(new SemanticValidator()));    
   
    
  }
  
  @FXML
  private void aqlKeyTyped(KeyEvent evt)
  {
    if(evt.getCode() == KeyCode.ENTER && evt.isControlDown()) {
      evt.consume();
      convert();
    }
  }

  @FXML
  public void convert()
  {
    jsonOutput.textProperty().set("converting...");
    convertButton.disableProperty().set(true);

    Task<String> task = new Task<String>()
    {
      @Override
      protected String call() throws Exception
      {
        QueryData queryData = parser.parse(aqlInput.textProperty().get(), null);
        
        return QueryToJSON.serializeQuery(queryData.getAlternatives(), queryData.getMetaData());
      }

      @Override
      protected void done()
      {
        super.failed();
        Platform.runLater(() -> {convertButton.disableProperty().set(false);});
      }

      @Override
      protected void failed()
      {
        super.failed();
        Platform.runLater(() -> {jsonOutput.textProperty().set("ERROR:\n" + getException().getMessage());});
      }
      
      @Override
      protected void succeeded()
      {
        super.succeeded();
        Platform.runLater(() ->
        {
          try
          {
            jsonOutput.textProperty().setValue(get());
          }
          catch (InterruptedException | ExecutionException ex)
          {
            log.error(null, ex);
          }
        });
        
      }
    };
    Thread thread = new Thread(task);
    thread.start();


  }

}
