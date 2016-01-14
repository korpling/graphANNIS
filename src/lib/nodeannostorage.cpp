/* 
 * File:   nodeannostorage.cpp
 * Author: thomas
 * 
 * Created on 14. Januar 2016, 13:53
 */

#include <annis/nodeannostorage.h>

#include <annis/stringstorage.h>

#include <fstream>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>
#include <boost/serialization/set.hpp>

using namespace annis;

NodeAnnoStorage::NodeAnnoStorage(StringStorage& strings)
: strings(strings)
{
}

bool NodeAnnoStorage::load(std::string dirPath)
{
  std::ifstream in;
  in.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.restore(in);
  in.close();

  in.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.restore(in);
  in.close();

  in.open(dirPath + "/nodeAnnoKeys.archive");
  boost::archive::binary_iarchive iaNodeAnnoKeys(in);
  iaNodeAnnoKeys >> nodeAnnoKeys;
  in.close();
}

bool NodeAnnoStorage::save(std::string dirPath)
{
  std::ofstream out;

  out.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.dump(out);
  out.close();

  out.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.dump(out);
  out.close();

  out.open(dirPath + "/nodeAnnoKeys.archive");
  boost::archive::binary_oarchive oaNodeAnnoKeys(out);
  oaNodeAnnoKeys << nodeAnnoKeys;
  out.close();
}

void NodeAnnoStorage::clear()
{
  nodeAnnotations.clear();
  inverseNodeAnnotations.clear();
}

NodeAnnoStorage::~NodeAnnoStorage()
{
}

