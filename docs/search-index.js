var searchIndex = JSON.parse('{\
"quack":{"doc":"The <em>quACK</em> is a data structure for being able to refer to …","t":"QQDIDDDDDDALLLLLLLLLLLLLLLLLLLLLLLLLLLLKLLLLLLKLLLLLLLLLLLLLLLLLLLLLLLLLLFKLLLLLLLLLLLLLKLLLLLKLLLLLLKLLLLLLLLLLLLMKLLLLLKLLLLLKLLLLLKLLLLLKLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLMMQGIDDQLLKLLLLLLLLLLLLLLLLLLLLFFFFLLLLLLKLLLLKLLLLKLLLLLLKLLLLKLLLLKLLLLLKLLLLLLLLKLLLLLLLLLLLLKLLLL","n":["Element","ModularElement","MontgomeryQuack","PowerSumQuack","PowerSumQuackU16","PowerSumQuackU32","PowerSumQuackU64","PowerTableQuack","StrawmanAQuack","StrawmanBQuack","arithmetic","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clone","clone","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","count","count","count","count","count","count","decode_by_factorization","decode_with_log","decode_with_log","decode_with_log","decode_with_log","decode_with_log","decode_with_log","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","global_config_set_max_power_sum_threshold","insert","insert","insert","insert","insert","insert","insert","into","into","into","into","into","into","into","last_value","last_value","last_value","last_value","last_value","last_value","new","new","new","new","new","new","new","remove","remove","remove","remove","remove","remove","serialize","serialize","serialize","serialize","serialize","serialize","serialize","sidekick_id","sub","sub","sub","sub","sub","sub","sub_assign","sub_assign","sub_assign","sub_assign","sub_assign","sub_assign","threshold","threshold","threshold","threshold","threshold","threshold","to_coeffs","to_coeffs","to_coeffs","to_coeffs","to_coeffs","to_coeffs","to_coeffs_preallocated","to_coeffs_preallocated","to_coeffs_preallocated","to_coeffs_preallocated","to_coeffs_preallocated","to_coeffs_preallocated","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","window","window_size","BigModulusType","CoefficientVector","ModularArithmetic","ModularInteger","MontgomeryInteger","SmallModulusType","add","add","add_assign","add_assign","add_assign","add_assign","add_assign","borrow","borrow","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","default","default","deserialize","deserialize","eq","eq","eq","eq","eval","eval_montgomery","eval_precompute","factor","fmt","fmt","from","from","into","into","inv","inv","inv","inv","inv","modulus","modulus","modulus","modulus","modulus","modulus_big","modulus_big","modulus_big","modulus_big","modulus_big","mul","mul","mul_assign","mul_assign","mul_assign","mul_assign","mul_assign","neg","neg","neg","neg","neg","new","new","new","new","new","new_do_conversion","pow","pow","pow","pow","pow","serialize","serialize","sub","sub","sub_assign","sub_assign","sub_assign","sub_assign","sub_assign","to_owned","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","value","value","value","value","value"],"q":[[0,"quack"],[175,"quack::arithmetic"],[274,"alloc::vec"],[275,"core::option"],[276,"core::result"],[277,"serde::de"],[278,"core::fmt"],[279,"core::fmt"],[280,"serde::ser"],[281,"core::any"],[282,"core::clone"],[283,"core::default"],[284,"serde::de"],[285,"core::marker"],[286,"core::fmt"]],"d":["The type of element that can be inserted in the quACK.","The modular version of the elements in the quACK.","64-bit power sum quACK using the Montgomery multiplication …","A quACK represented by a threshold number of power sums.","16-bit power sum quACK.","32-bit power sum quACK.","64-bit power sum quACK.","16-bit power sum quACK using the precomputation …","Strawman quACK implementation that echoes every packet …","Strawman quACK implementation that echoes a sliding window …","Efficient modular arithmetic and polynomial evaluation.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The number of elements represented by the quACK.","","","","","","Decode the elements in the quACK by factorization.","Decode the elements in the log that in the quACK.","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","The multiplicative modular inverses of the integers up to …","Insert an element in the quACK.","","","","","","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","The last element inserted in the quACK, if known.","","","","","","Creates a new power sum quACK that can decode at most …","","","","","","","Remove an element in the quACK. Does not validate that the …","","","","","","","","","","","","","","Similar to sub_assign but returns the difference as a new …","","","","","","Subtracts another power sum quACK from this power sum …","Subtracts another power sum quACK from this power sum …","","","","","The maximum number of elements that can be decoded by the …","","","","","","Convert the <code>n</code> modular power sums that represent the …","Convert the <code>n</code> modular power sums that represent the …","","","","","Similar to to_coeffs but reuses the same vector allocation …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The next largest unsigned integer type that fits elements …","The coefficient vector defines a univariate polynomial …","Arithmetic operations and other properties of the modular …","An element in the finite field with integers modulo a …","A 64-bit finite field element in Montgomery form.","The smallest unsigned integer type that fits elements in …","Performs the <code>+</code> operation in the finite field.","Performs the <code>+</code> operation in the finite field.","Performs the <code>+=</code> operation in the finite field.","","","","Performs the <code>+=</code> operation in the finite field.","","","","","","","","","","","","","","","","","Evaluate the univariate polynomial at <code>x</code> in a modular …","Evaluate the univariate polynomial at <code>x</code>, assuming that <code>x</code> …","Evaluate the univariate polynomial at <code>x</code> in a 16-bit …","Factor the univariate polynomial using 32-bit modular …","","","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","The modular multiplicative inverse of the element.","","","","","The modulus prime of the finite field.","The modulus prime of the finite field, …","The modulus prime of the finite field, <code>65521</code>, as a <code>u16</code>. …","The modulus prime of the finite field, <code>4_294_967_291</code>, as a …","The original prime modulus, <code>18_446_744_073_709_551_557</code>, as …","The modulus prime of the finite field, as a larger …","The modulus prime of the finite field, <code>65521</code>, as a <code>u32</code>. …","The modulus prime of the finite field, <code>4_294_967_291</code>, as a …","The modulus prime of the finite field, …","The original prime modulus, <code>18_446_744_073_709_551_557</code>, as …","Performs the <code>*</code> operation in the finite field.","Performs the <code>*</code> operation in the finite field.","Performs the <code>*=</code> operation in the finite field.","","","","Performs the <code>*=</code> operation in the finite field.","Performs the unary <code>-</code> operation in the finite field.","","","","","Creates a new element in the finite field.","","","","Creates a new Montgomery integer, assuming the provided …","Create a new Montgomery integer, doing the conversion from …","Raises the element to the <code>power</code>-th power in the finite …","","","","","","","Performs the <code>-</code> operation in the finite field.","Performs the <code>-</code> operation in the finite field.","Performs the <code>-=</code> operation in the finite field.","","","","Performs the <code>-=</code> operation in the finite field.","","","","","","","","","The integer value of the element, where …","","","",""],"i":[35,35,0,0,0,0,0,0,0,0,0,1,2,3,4,5,6,7,1,2,3,4,5,6,7,1,2,3,4,5,6,7,1,2,3,4,5,6,7,35,1,2,3,4,5,1,35,1,2,3,4,5,1,2,3,4,5,6,7,1,2,3,4,5,6,7,1,2,3,4,5,6,7,0,35,1,2,3,4,5,6,1,2,3,4,5,6,7,35,1,2,3,4,5,35,1,2,3,4,5,6,35,1,2,3,4,5,1,2,3,4,5,6,7,7,35,1,2,3,4,5,35,1,2,3,4,5,35,1,2,3,4,5,35,1,2,3,4,5,35,1,2,3,4,5,1,2,3,4,5,6,7,1,2,3,4,5,6,7,1,2,3,4,5,6,7,1,2,3,4,5,6,7,6,6,36,0,0,0,0,36,36,36,36,23,23,23,25,23,25,23,25,23,25,23,25,23,25,23,25,23,23,25,25,0,0,0,0,23,25,23,25,23,25,36,23,23,23,25,36,23,23,23,25,36,23,23,23,25,36,36,36,23,23,23,25,36,23,23,23,25,36,23,23,23,25,25,36,23,23,23,25,23,25,36,36,36,23,23,23,25,23,25,23,25,23,25,23,25,36,23,23,23,25],"f":[0,0,0,0,0,0,0,0,0,0,0,[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[1,1],[2,2],[3,3],[4,4],[5,5],[6,6],[7,7],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[-1,9,[]],[1,9],[2,9],[3,9],[4,9],[5,9],[1,[[11,[[10,[9]]]]]],[[-1,12],10,[]],[[1,12],10],[[2,12],10],[[3,12],10],[[4,12],10],[[5,[12,[13]]],[[10,[13]]]],[-1,[[14,[1]]],15],[-1,[[14,[2]]],15],[-1,[[14,[3]]],15],[-1,[[14,[4]]],15],[-1,[[14,[5]]],15],[-1,[[14,[6]]],15],[-1,[[14,[7]]],15],[[1,16],17],[[2,16],17],[[3,16],17],[[4,16],17],[[5,16],17],[[6,16],17],[[7,16],17],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[18,8],[-1,8,[]],[1,8],[2,8],[3,8],[4,8],[5,8],[[6,9],8],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,11,[]],[1,11],[2,11],[3,11],[4,11],[5,11],[18,-1,19],[18,1],[18,2],[18,3],[18,4],[18,5],[18,6],[-1,8,[]],[1,8],[2,8],[3,8],[4,8],[5,8],[[1,-1],14,20],[[2,-1],14,20],[[3,-1],14,20],[[4,-1],14,20],[[5,-1],14,20],[[6,-1],14,20],[[7,-1],14,20],0,[[-1,-1],-1,[]],[[1,1],1],[[2,2],2],[[3,3],3],[[4,4],4],[[5,5],5],[[-1,-1],8,[]],[[1,1],8],[[2,2],8],[[3,3],8],[[4,4],8],[[5,5],8],[-1,18,[]],[1,18],[2,18],[3,18],[4,18],[5,18],[-1,21,[]],[1,21],[2,21],[3,21],[4,21],[5,21],[[-1,21],8,[]],[[1,21],8],[[2,21],8],[[3,21],8],[[4,21],8],[[5,21],8],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,22,[]],[-1,22,[]],[-1,22,[]],[-1,22,[]],[-1,22,[]],[-1,22,[]],[-1,22,[]],0,0,0,0,0,0,0,0,[[-1,-1],-1,19],[[-1,-1],-1,19],[[-1,-1],8,[]],[[[23,[9]],[23,[9]]],8],[[[23,[13]],[23,[13]]],8],[[[23,[24]],[23,[24]]],8],[[25,25],8],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[[[23,[-1]]],[[23,[-1]]],26],[25,25],[[-1,-2],8,[],[]],[[-1,-2],8,[],[]],[[],[[23,[-1]]],27],[[],25],[-1,[[14,[[23,[-2]]]]],15,28],[-1,[[14,[25]]],15],[[[23,[-1]],-1],29,30],[[[23,[-1]],[23,[-1]]],29,30],[[25,25],29],[[25,13],29],[[[21,[[23,[-1]]]]],[[23,[-1]]],31],[[[10,[25]],13],25],[[[21,[[23,[24]]]],24],[[23,[24]]]],[[[21,[[23,[9]]]]],[[14,[[10,[9]],32]]]],[[[23,[-1]],16],17,33],[[25,16],17],[-1,-1,[]],[-1,-1,[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-1,[]],[[[23,[9]]],[[23,[9]]]],[[[23,[13]]],[[23,[13]]]],[[[23,[24]]],[[23,[24]]]],[25,25],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[-1,-1],-1,19],[[-1,-1],-1,19],[[-1,-1],8,[]],[[[23,[24]],[23,[24]]],8],[[[23,[9]],[23,[9]]],8],[[[23,[13]],[23,[13]]],8],[[25,25],8],[-1,-1,[]],[[[23,[9]]],[[23,[9]]]],[[[23,[24]]],[[23,[24]]]],[[[23,[13]]],[[23,[13]]]],[25,25],[[],-1,[]],[[],[[23,[9]]]],[[],[[23,[24]]]],[[],[[23,[13]]]],[13,25],[13,25],[-1,-1,[]],[[[23,[24]]],[[23,[24]]]],[[[23,[13]]],[[23,[13]]]],[[[23,[9]]],[[23,[9]]]],[[25,13],25],[[[23,[-1]],-2],14,34,20],[[25,-1],14,20],[[-1,-1],-1,19],[[-1,-1],-1,19],[[-1,-1],8,[]],[[[23,[9]],[23,[9]]],8],[[[23,[24]],[23,[24]]],8],[[[23,[13]],[23,[13]]],8],[[25,25],8],[-1,-2,[],[]],[-1,-2,[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,22,[]],[-1,22,[]],[-1,[],[]],[[[23,[9]]]],[[[23,[24]]]],[[[23,[13]]]],[25]],"c":[],"p":[[3,"PowerSumQuackU32",0],[3,"PowerSumQuackU64",0],[3,"PowerSumQuackU16",0],[3,"PowerTableQuack",0],[3,"MontgomeryQuack",0],[3,"StrawmanBQuack",0],[3,"StrawmanAQuack",0],[15,"tuple"],[15,"u32"],[3,"Vec",274],[4,"Option",275],[15,"slice"],[15,"u64"],[4,"Result",276],[8,"Deserializer",277],[3,"Formatter",278],[6,"Result",278],[15,"usize"],[8,"Sized",279],[8,"Serializer",280],[6,"CoefficientVector",175],[3,"TypeId",281],[3,"ModularInteger",175],[15,"u16"],[3,"MontgomeryInteger",175],[8,"Clone",282],[8,"Default",283],[8,"Deserialize",277],[15,"bool"],[8,"PartialEq",284],[8,"Copy",279],[3,"String",285],[8,"Debug",278],[8,"Serialize",280],[8,"PowerSumQuack",0],[8,"ModularArithmetic",175]],"b":[[184,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[185,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[186,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[200,"impl-PartialEq%3CT%3E-for-ModularInteger%3CT%3E"],[201,"impl-PartialEq-for-ModularInteger%3CT%3E"],[202,"impl-PartialEq-for-MontgomeryInteger"],[203,"impl-PartialEq%3Cu64%3E-for-MontgomeryInteger"],[215,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[216,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[217,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[220,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[221,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[222,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[225,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[226,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[227,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[232,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[233,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[234,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[237,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[238,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[239,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[242,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[243,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[244,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[248,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[249,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[250,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[257,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[258,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[259,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"],[270,"impl-ModularArithmetic-for-ModularInteger%3Cu32%3E"],[271,"impl-ModularArithmetic-for-ModularInteger%3Cu16%3E"],[272,"impl-ModularArithmetic-for-ModularInteger%3Cu64%3E"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
