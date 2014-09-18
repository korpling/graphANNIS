#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <humblelogging/api.h>

#include "helper.h"
#include "edgedb/fallbackedgedb.h"

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB()
{
  addDefaultStrings();
}

bool DB::load(string dirPath)
{
  typedef std::map<Component, EdgeDB*, compComponent>::const_iterator EDBIt;
  clear();
  addDefaultStrings();

  strings.load(dirPath);

  ifstream in;

  in.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.restore(in);
  in.close();

  in.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.restore(in);
  in.close();

  boost::filesystem::directory_iterator fileEndIt;

  for(unsigned int componentType = (unsigned int) ComponentType::COVERAGE;
      componentType < (unsigned int) ComponentType::ComponentType_MAX; componentType++)
  {
    const boost::filesystem::path componentPath(dirPath + "/edgedb/"
                                                + ComponentTypeToString((ComponentType) componentType));

    if(boost::filesystem::is_directory(componentPath))
    {
      // get all the namespaces/layers
      boost::filesystem::directory_iterator itLayers(componentPath);
      while(itLayers != fileEndIt)
      {
        const boost::filesystem::path layerPath = *itLayers;

        // try to load the component with the empty name

        EdgeDB* edbEmptyName = createEdgeDBForComponent((ComponentType) componentType,
                                               layerPath.filename().string(), "");
        edbEmptyName->load(layerPath.string());

        // also load all named components
        boost::filesystem::directory_iterator itNamedComponents(layerPath);
        while(itNamedComponents != fileEndIt)
        {
          const boost::filesystem::path namedComponentPath = *itNamedComponents;
          if(boost::filesystem::is_directory(namedComponentPath))
          {
            // try to load the named component
            EdgeDB* edbNamed = createEdgeDBForComponent((ComponentType) componentType,
                                                   layerPath.filename().string(),
                                                   namedComponentPath.filename().string());
            edbNamed->load(namedComponentPath.string());
          }
          itNamedComponents++;
        } // end for each file/directory in layer directory
        itLayers++;
      } // for each layers
    }
  } // end for each component

  // TODO: return false on failure
  return true;
}

bool DB::save(string dirPath)
{
  typedef std::map<Component, EdgeDB*, compComponent>::const_iterator EdgeDBIt;

  boost::filesystem::create_directories(dirPath);

  strings.save(dirPath);

  ofstream out;

  out.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.dump(out);
  out.close();

  out.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.dump(out);
  out.close();

  // save each edge db separately
  string edgeDBParent = dirPath + "/edgedb";
  for(EdgeDBIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    string finalPath;
    if(c.name == NULL)
    {
      finalPath = edgeDBParent + "/" + ComponentTypeToString(c.type) + "/" + c.layer;
    }
    else
    {
      finalPath = edgeDBParent + "/" + ComponentTypeToString(c.type) + "/" + c.layer + "/" + c.name;
    }
    boost::filesystem::create_directories(finalPath);
    it->second->save(finalPath);
  }


  // TODO: return false on failure
  return true;
}

bool DB::loadRelANNIS(string dirPath)
{
  clear();
  addDefaultStrings();

  if(loadRelANNISNode(dirPath) == false)
  {
    return false;
  }

  string componentTabPath = dirPath + "/component.tab";
  HL_INFO(logger, (boost::format("loading %1%") % componentTabPath).str());

  ifstream in;
  vector<string> line;

  in.open(componentTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, EdgeDB*> componentToEdgeDB;
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t componentID = uint32FromString(line[0]);
    if(line[1] != "NULL")
    {
      EdgeDB* edb = createEdgeDBForComponent(line[1], line[2], line[3]);
      componentToEdgeDB[componentID] = edb;
    }
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToEdgeDB);


  return result;
}


bool DB::loadRelANNISNode(string dirPath)
{
  // maps a token index to an node ID
  map<TokenIndex, uint32_t, compTokenIndex> tokenByIndex;

  string nodeTabPath = dirPath + "/node.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeTabPath).str());

  ifstream in;
  in.open(nodeTabPath, ifstream::in);
  if(!in.good())
  {
    HL_ERROR(logger, "Can't find node.tab");
    return false;
  }
  vector<string> line;
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t nodeNr;
    stringstream nodeNrStream(line[0]);
    nodeNrStream >> nodeNr;

    bool hasSegmentations = line.size() > 10;
    string tokenIndexRaw = line[7];
    uint32_t textID = uint32FromString(line[1]);
    Annotation nodeNameAnno;
    nodeNameAnno.ns = strings.add(annis_ns);
    nodeNameAnno.name = strings.add("node_name");
    nodeNameAnno.val = strings.add(line[4]);
    addNodeAnnotation(nodeNr, nodeNameAnno);
    if(tokenIndexRaw != "NULL")
    {
      string span = hasSegmentations ? line[12] : line[9];

      Annotation tokAnno;
      tokAnno.ns = strings.add(annis_ns);
      tokAnno.name = strings.add("tok");
      tokAnno.val = strings.add(span);
      addNodeAnnotation(nodeNr, tokAnno);

      TokenIndex index;
      index.tokenIndex = uint32FromString(tokenIndexRaw);
      index.textID = textID;

      tokenByIndex[index] = nodeNr;
    }
  }

  in.close();

  // iterate over all token by their order and add an explicit edge
  HL_DEBUG(logger, (boost::format("tokenByIndex size: %1%") % tokenByIndex.size()).str());
  if(tokenByIndex.size() > 1)
  {
    EdgeDB* edb = createEdgeDBForComponent(ComponentType::ORDERING, annis_ns, "tok");
    map<TokenIndex, uint32_t, compTokenIndex>::const_iterator tokenIt = tokenByIndex.begin();
    uint32_t lastNodeNr = tokenIt->second;
    uint32_t lastTextID = tokenIt->first.textID;

    while(tokenIt != tokenByIndex.end())
    {
      uint32_t currentTextID = tokenIt->first.textID;
      if(currentTextID == lastTextID)
      {
        edb->addEdge(constructEdge(lastNodeNr, tokenIt->second));
      }
      lastTextID = currentTextID;
      lastNodeNr = tokenIt->second;

      tokenIt++;
    }
  }


  string nodeAnnoTabPath = dirPath + "/node_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeAnnoTabPath).str());

  in.open(nodeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  while((line = nextCSV(in)).size() > 0)
  {
    u_int32_t nodeNr = uint32FromString(line[0]);
    Annotation anno;
    anno.ns = strings.add(line[1]);
    anno.name = strings.add(line[2]);
    anno.val = strings.add(line[3]);
    addNodeAnnotation(nodeNr, anno);
  }

  in.close();
  return true;
}


bool DB::loadRelANNISRank(const string &dirPath,
                          const map<uint32_t, EdgeDB*>& componentToEdgeDB)
{
  typedef stx::btree_map<uint32_t, uint32_t>::const_iterator UintMapIt;
  typedef map<uint32_t, EdgeDB*>::const_iterator ComponentIt;
  bool result = true;

  ifstream in;
  string rankTabPath = dirPath + "/rank.tab";
  HL_INFO(logger, (boost::format("loading %1%") % rankTabPath).str());

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  // first run: collect all pre-order values for a node
  stx::btree_map<uint32_t, uint32_t> pre2NodeID;
  map<uint32_t, Edge> pre2Edge;

  while((line = nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert2(uint32FromString(line[0]),uint32FromString(line[2]));
  }

  in.close();

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, EdgeDB* > pre2EdgeDB;

  // second run: get the actual edges
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t parent = uint32FromString(line[4]);
    UintMapIt it = pre2NodeID.find(parent);
    if(it != pre2NodeID.end())
    {
      // find the responsible edge database by the component ID
      ComponentIt itEdb = componentToEdgeDB.find(uint32FromString(line[3]));
      if(itEdb != componentToEdgeDB.end())
      {
        EdgeDB* edb = itEdb->second;
        Edge edge = constructEdge(uint32FromString(line[2]), it->second);

        edb->addEdge(edge);
        pre2Edge[uint32FromString(line[0])] = edge;
        pre2EdgeDB[uint32FromString(line[0])] = edb;
      }
    }
    else
    {
      result = false;
    }
  }

  in.close();


  if(result)
  {

    result = loadEdgeAnnotation(dirPath, pre2EdgeDB, pre2Edge);
  }

  return result;
}


bool DB::loadEdgeAnnotation(const string &dirPath,
                            const map<uint32_t, EdgeDB* >& pre2EdgeDB,
                            const map<uint32_t, Edge>& pre2Edge)
{

  bool result = true;

  ifstream in;
  string edgeAnnoTabPath = dirPath + "/edge_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % edgeAnnoTabPath).str());

  in.open(edgeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t pre = uint32FromString(line[0]);
    map<uint32_t, EdgeDB*>::const_iterator itDB = pre2EdgeDB.find(pre);
    map<uint32_t, Edge>::const_iterator itEdge = pre2Edge.find(pre);
    if(itDB != pre2EdgeDB.end() && itEdge != pre2Edge.end())
    {
      EdgeDB* e = itDB->second;
      Annotation anno;
      anno.ns = strings.add(line[1]);
      anno.name = strings.add(line[2]);
      anno.val = strings.add(line[3]);
      if(e != NULL)
      {
        e->addEdgeAnnotation(itEdge->second, anno);
      }
    }
    else
    {
      result = false;
    }
  }

  in.close();

  return result;
}

void DB::clear()
{
  strings.clear();
  nodeAnnotations.clear();
}

void DB::addDefaultStrings()
{
  strings.add(annis_ns);
  strings.add("");
  strings.add("tok");
  strings.add("node_name");
}

EdgeDB *DB::createEdgeDBForComponent(const string &shortType, const string &layer, const string &name)
{
  // fill the component variable
  ComponentType ctype;
  if(shortType == "c")
  {
    ctype = ComponentType::COVERAGE;
  }
  else if(shortType == "d")
  {
    ctype = ComponentType::DOMINANCE;
  }
  else if(shortType == "p")
  {
    ctype = ComponentType::POINTING;
  }
  else if(shortType == "o")
  {
    ctype = ComponentType::ORDERING;
  }
  else
  {
    throw("Unknown component type \"" + shortType + "\"");
  }
  return createEdgeDBForComponent(ctype, layer, name);

}

EdgeDB *DB::createEdgeDBForComponent(ComponentType ctype, const string &layer, const string &name)
{
  Component c = constructComponent(ctype, layer, name);

  // check if there is already an edge DB for this component
  map<Component,EdgeDB*,compComponent>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB == edgeDatabases.end())
  {

    // TODO: decide which implementation to use
    EdgeDB* edgeDB = new FallbackEdgeDB(strings, c);

    // register the used implementation
    edgeDatabases.insert(pair<Component,EdgeDB*>(c,edgeDB));
    return edgeDB;
  }
  else
  {
    return itDB->second;
  }
}

bool DB::hasNode(uint32_t id)
{
  stx::btree_multimap<uint32_t, Annotation>::const_iterator itNode = nodeAnnotations.find(id);
  if(itNode == nodeAnnotations.end())
  {
    return false;
  }
  else
  {
    return true;
  }
}

/*
vector<Edge> DB::getInEdges(uint32_t nodeID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.target = nodeID;
  keyUpper.target = nodeID;

  keyLower.source = numeric_limits<uint32_t>::min();
  keyLower.component = numeric_limits<uint32_t>::min();

  keyUpper.source = numeric_limits<uint32_t>::max();
  keyUpper.component = numeric_limits<uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}
*/

string DB::info()
{
  typedef map<Component, EdgeDB*, compComponent>::const_iterator EdgeDBIt;
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnotations.size() << endl
      << "Number of strings in storage: " << strings.size() << endl;

  for(EdgeDBIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    const EdgeDB* edb = it->second;
    ss << "Component " << ComponentTypeToString(c.type) << "|" << c.layer
       << "|" << c.name << ": " << edb->numberOfEdges() << " edges and "
       << edb->numberOfEdgeAnnotations() << " annotations" << endl;
  }

  return ss.str();
}

/*
vector<Edge> DB::getEdgesBetweenNodes(uint32_t sourceID, uint32_t targetID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.source = sourceID;
  keyUpper.source = sourceID;
  keyLower.target = targetID;
  keyUpper.target = targetID;

  keyLower.component = numeric_limits<uint32_t>::min();
  keyUpper.component = numeric_limits<uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}
*/

vector<Annotation> DB::getNodeAnnotationsByID(const uint32_t& id)
{
  typedef stx::btree_multimap<uint32_t, Annotation>::const_iterator AnnoIt;

  vector<Annotation> result;

  pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);

  for(AnnoIt itAnnos = itRange.first;
      itAnnos != itRange.second; itAnnos++)
  {
    result.push_back(itAnnos->second);
  }

  return result;
}

std::vector<Component> DB::getDirectConnected(const Edge &edge)
{
  std::vector<Component> result;
  map<Component, EdgeDB*>::const_iterator itEdgeDB = edgeDatabases.begin();

  while(itEdgeDB != edgeDatabases.end())
  {
    EdgeDB* edb = itEdgeDB->second;
    if(edb != NULL)
    {
      if(edb->isConnected(edge))
      {
        result.push_back(itEdgeDB->first);
      }
    }
    itEdgeDB++;
  }

  return result;
}

const EdgeDB* DB::getEdgeDB(const Component &component)
{
  map<Component, EdgeDB*>::const_iterator itEdgeDB = edgeDatabases.find(component);
  if(itEdgeDB != edgeDatabases.end())
  {
    return itEdgeDB->second;
  }
  return NULL;
}

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,EdgeDB*>::const_iterator it = edgeDatabases.find(component);
  if(it != edgeDatabases.end() && it->second != NULL)
  {
    EdgeDB* edb = it->second;
    return edb->getEdgeAnnotations(edge);
  }

  return vector<Annotation>();

}

DB::~DB()
{
  for(auto& ed : edgeDatabases)
  {
    delete ed.second;
  }
}
