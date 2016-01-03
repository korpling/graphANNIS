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

import java.io.IOException;
import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.image.Image;
import javafx.stage.Stage;

/**
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class Main extends Application
{

  @Override
  public void start(Stage primaryStage) throws IOException
  {

    primaryStage.getIcons().addAll(
      new Image(getClass().getResourceAsStream("icon128.png")),
      new Image(getClass().getResourceAsStream("icon64.png")),
      new Image(getClass().getResourceAsStream("icon32.png")));

    FXMLLoader loader = new FXMLLoader(QueryConverterController.class.
      getResource(
        "QueryConverter.fxml"));
    Parent root = loader.load();

    Scene scene = new Scene(root);
    QueryConverterController controller = loader.getController();
    controller.initializeAccelerators(scene);

    primaryStage.setTitle("AQL Benchmark Generator");
    primaryStage.setScene(scene);

    primaryStage.show();

  }

  /**
   * @param args the command line arguments
   */
  public static void main(String[] args)
  {
    launch(args);
  }

}
