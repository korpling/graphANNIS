#include <iostream>
#include <stx/btree_map>
#include <cstdint>

using namespace std;

int main()
{
  stx::btree_map<uint32_t,uint32_t> testmap;
  testmap[234] = 23;
  cout << "Hello World! " << testmap[234] << endl;
  return 0;
}

