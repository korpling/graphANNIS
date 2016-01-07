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

import com.google.common.io.Files;
import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.ResourceBundle;
import javafx.beans.property.SimpleObjectProperty;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Parent;
import javafx.scene.control.Alert;
import javafx.scene.control.ButtonType;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.control.TextField;
import javafx.scene.control.cell.PropertyValueFactory;
import javafx.scene.text.Text;
import javafx.stage.FileChooser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * FXML Controller class
 *
 * @author thomas
 */
public class QuerySetViewController implements Initializable
{

  private final Logger log = LoggerFactory.getLogger(
    QuerySetViewController.class);

  @FXML
  private Parent root;

  private FileChooser chooser = new FileChooser();

  private FileChooser.ExtensionFilter logFilter = new FileChooser.ExtensionFilter(
    "Query log (*.log)", "*.log");

  @FXML
  private TableView<Query> tableView;

  @FXML
  private TableColumn<Query, String> aqlColumn;
  
  @FXML
  private TableColumn<Query, String> corpusColumn;
  
  @FXML
  private TableColumn<Query, Long> execTimeColumn;
  
  @FXML
  private TableColumn<Query, Long> nrResultsColumn;
  
  @FXML
  private TextField corpusFilter;

  /**
   * Initializes the controller class.
   */
  @Override
  public void initialize(URL url, ResourceBundle rb)
  {

    aqlColumn.setCellValueFactory(new PropertyValueFactory<>("aql"));
    corpusColumn.setCellValueFactory(new PropertyValueFactory<>("corpus"));
    
    execTimeColumn.setCellValueFactory((TableColumn.CellDataFeatures<Query, Long> param) -> new SimpleObjectProperty<>(
        param.getValue().getExecutionTime().orElse(-1l)));
    
    nrResultsColumn.setCellValueFactory((TableColumn.CellDataFeatures<Query, Long> param) -> new SimpleObjectProperty<>(
        param.getValue().getCount().orElse(-1l)));

    

    aqlColumn.setCellFactory((TableColumn<Query, String> param)
      -> 
      {
        final TableCell cell = new TableCell()
        {
          private Text text;

          @Override
          public void updateItem(Object item, boolean empty)
          {
            super.updateItem(item, empty);
            if (!isEmpty())
            {
              text = new Text(item.toString());
              text.wrappingWidthProperty().bind(widthProperty());
              setGraphic(text);
            }
          }
        };
        return cell;

    });
  }

  @FXML
  public void loadQueryLog(ActionEvent evt)
  {
    chooser.setTitle("Open Query Log");
    chooser.getExtensionFilters().add(logFilter);
    chooser.setSelectedExtensionFilter(logFilter);

    File selectedFile = chooser.showOpenDialog(root.getScene().getWindow());
    if (selectedFile != null)
    {
      try
      {
        QuerySet queries = Files.readLines(selectedFile, StandardCharsets.UTF_8,
          new QueryLogParser());
        
        tableView.itemsProperty().get().clear();
        tableView.itemsProperty().get().addAll(queries.getAll());

      }
      catch (IOException ex)
      {
        log.error(null, ex);
        new Alert(Alert.AlertType.ERROR, "Could not parse file: " + ex.
          getMessage(), ButtonType.OK).showAndWait();

      }
    }
  }

}
