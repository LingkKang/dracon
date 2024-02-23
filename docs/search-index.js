var searchIndex = new Map(JSON.parse('[\
["raft",{"doc":"The Raft consensus algorithm implementation.","t":"CPPPFGNNNNNNNNNNNNNONNNNNN","n":["node","Candidate","Follower","Leader","Node","Status","borrow","borrow","borrow_mut","borrow_mut","default","from","from","into","into","is_leader","is_leader","new","status","status","try_from","try_from","try_into","try_into","type_id","type_id"],"q":[[0,"raft"],[1,"raft::node"],[26,"core::result"],[27,"core::any"]],"d":["In Raft, each server is represented as a node <code>Node</code>.","Candidate status.","Follower status. The default status of a new node.","Leader status.","","All possible status (states) of a Raft node.","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Check if the node is a leader.","","Create a new node with <code>Status::Follower</code> status.","Get the current status of the node.","","","","","","",""],"i":[0,3,3,3,0,0,1,3,1,3,1,1,3,1,3,1,3,1,1,1,1,3,1,3,1,3],"f":[0,0,0,0,0,0,[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[[],1],[-1,-1,[]],[-1,-1,[]],[-1,-2,[],[]],[-1,-2,[],[]],[1,2],[3,2],[[],1],[1,3],0,[-1,[[4,[-2]]],[],[]],[-1,[[4,[-2]]],[],[]],[-1,[[4,[-2]]],[],[]],[-1,[[4,[-2]]],[],[]],[-1,5,[]],[-1,5,[]]],"c":[],"p":[[5,"Node",1],[1,"bool"],[6,"Status",1],[6,"Result",26],[5,"TypeId",27]],"b":[]}]\
]'));
if (typeof exports !== 'undefined') exports.searchIndex = searchIndex;
else if (window.initSearch) window.initSearch(searchIndex);
