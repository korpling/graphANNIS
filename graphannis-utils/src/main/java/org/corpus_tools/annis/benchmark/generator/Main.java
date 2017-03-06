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
package org.corpus_tools.annis.benchmark.generator;

import java.awt.Font;
import java.awt.GraphicsDevice;
import java.awt.GraphicsEnvironment;
import java.io.IOException;
import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.image.Image;
import javafx.stage.Screen;
import javafx.stage.Stage;
import javax.swing.UIDefaults;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.bridge.SLF4JBridgeHandler;

/**
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class Main extends Application
{

  private Logger log = LoggerFactory.getLogger(Main.class);

  @Override
  public void start(Stage primaryStage) throws IOException
  {
    SLF4JBridgeHandler.removeHandlersForRootLogger();;
    SLF4JBridgeHandler.install();

    log.info("Starting AQL Benchmark Generator");

    primaryStage.getIcons().addAll(
      new Image(getClass().getResourceAsStream("icon128.png")),
      new Image(getClass().getResourceAsStream("icon64.png")),
      new Image(getClass().getResourceAsStream("icon32.png")));

    FXMLLoader loader = new FXMLLoader(MainController.class.
      getResource(
        "Main.fxml"));
    Parent root = loader.load();

    if(Screen.getPrimary().getBounds().getWidth() > 2000)
    {
      root.setStyle("-fx-font-size:18;");
    }
    
    Scene scene = new Scene(root);
    MainController controller = loader.getController();
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
