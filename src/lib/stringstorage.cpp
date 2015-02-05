#include "stringstorage.h"
#include <fstream>
#include "helper.h"

#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/serialization/map.hpp>

#include <re2/re2.h>

using namespace annis;
using namespace std;

StringStorage::StringStorage()
{
}

std::set<std::uint32_t> StringStorage::findRegex(const string &str) const
{
  using ItType = map<string, uint32_t>::const_iterator;
  std::set<std::uint32_t> result;

  RE2 re(str, RE2::Quiet);
  if(re.ok())
  {
    // get the size of the last element so we know how large our prefix needs to be
    size_t prefixSize = 10;
    const std::string& lastString = stringStorageByValue.rbegin()->first;
    size_t lastStringSize = lastString.size()+1;
    if(lastStringSize > prefixSize)
    {
      prefixSize = lastStringSize;
    }

    std::string minPrefix;
    std::string maxPrefix;
    re.PossibleMatchRange(&minPrefix, &maxPrefix, prefixSize);

    ItType upperBound = stringStorageByValue.upper_bound(maxPrefix);

    for(ItType it=stringStorageByValue.lower_bound(minPrefix);
        it != upperBound; it++)
    {
      if(RE2::FullMatch(it->first, re))
      {
        result.insert(it->second);
      }
    }
  }

  return result;
}

uint32_t StringStorage::add(const string &str)
{
  typedef map<string, uint32_t>::const_iterator ItType;
  ItType it = stringStorageByValue.find(str);
  if(it == stringStorageByValue.end())
  {
    // non-existing
    uint32_t id = 1; // since 0 is taken as ANY value begin with 1
    if(stringStorageByID.size() > 0)
    {
      id = ((stringStorageByID.rbegin())->first)+1;
    }
    stringStorageByID.insert(pair<uint32_t, string>(id, str));
    stringStorageByValue.insert(pair<string, uint32_t>(str, id));
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

  in.open(dirPath + "/stringStorageByID.archive", ios::binary);
  boost::archive::binary_iarchive iaByID(in);
  iaByID >> stringStorageByID;
  in.close();

  in.open(dirPath + "/stringStorageByValue.archive", ios::binary);
  boost::archive::binary_iarchive iaByValue(in);
  iaByValue >> stringStorageByValue;
  in.close();

  return true;
}


bool StringStorage::save(const std::string& dirPath)
{
  ofstream out;

  out.open(dirPath + "/stringStorageByID.archive", ios::binary);
  boost::archive::binary_oarchive oaByID(out);
  oaByID << stringStorageByID;
  out.close();

  out.open(dirPath + "/stringStorageByValue.archive", ios::binary);
  boost::archive::binary_oarchive oaByValue(out);
  oaByValue << stringStorageByValue;
  out.close();


  return true;
}


