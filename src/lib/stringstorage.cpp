#include "stringstorage.h"
#include <fstream>
#include "helper.h"

using namespace annis;
using namespace std;

StringStorage::StringStorage()
{
}

uint32_t StringStorage::add(const string &str)
{
  typedef stx::btree_map<string, uint32_t>::const_iterator ItType;
  ItType it = stringStorageByValue.find(str);
  if(it == stringStorageByValue.end())
  {
    // non-existing
    uint32_t id = 1; // since 0 is taken as ANY value begin with 1
    if(stringStorageByID.size() > 0)
    {
      id = ((stringStorageByID.rbegin())->first)+1;
    }
    stringStorageByID.insert2(id, str);
    stringStorageByValue.insert2(str, id);
    return id;
  }
  else
  {
    // already existing, return the original ID
    return it->second;
  }
}

void StringStorage::clear()
{

  stringStorageByID.clear();
  stringStorageByValue.clear();

}

bool StringStorage::load(const string &dirPath)
{

  ifstream in;
  // load the strings from CSV
  in.open(dirPath + "/strings.list");
  vector<string> line;
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t id = uint32FromString(line[0]);
    const std::string& val = line.size() > 1 ? line[1] : "";
    stringStorageByID.insert2(id, val);
    stringStorageByValue.insert2(val, id);
  }
  in.close();
  return true;
}


bool StringStorage::save(const std::string& dirPath)
{

  typedef stx::btree<uint32_t, string>::const_iterator StringStorageIt;

  ofstream out;

  // save the strings to CSV
  out.open(dirPath + "/strings.list");
  StringStorageIt it = stringStorageByID.begin();
  while(it != stringStorageByID.end())
  {
    vector<string> line;
    line.push_back(stringFromUInt32(it->first));
    line.push_back(it->second);
    writeCSVLine(out, line);
    it++;
    if(it != stringStorageByID.end())
    {
      out << "\n";
    }
  }
  out.close();
  return true;
}


