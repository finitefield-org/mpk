"""Fixed typed control labels for the actual count/variable/signed proof additions."""
def names():
 expected=set()
 for n in ['CountBitComplement','CountPrefixComplementStep','RightStrictGap','RightRemainderComposeBound','LeftCompose','RightSaturationCoefficient','RightSaturatedConservation','LeftSignedBalance','RightSignedBalance']:expected.add('False-'+n)
 for n,i in [('CountPrefixComplementStep',5),('CountPrefixComplementStep',6),('CountComplementBound',3),('RightSaturationCoefficient',9),('RightSaturatedRemainderBound',9),('RightSaturatedRemainderBound',11)]:expected.add(f'Fake-{n}-{i}')
 for d in range(3,7):
  expected.add(f'False-CountBelowWidth-{d}')
  for n in ['CountLargePower','CountSmallValue']:expected.add(f'Fake-{n}-{d}')
  expected.add(f'Wrong-CountExcess-{d}')
 for w in [8,16,32,64]:
  for n in ['UnsignedLeftConservation','SignedLeftRefinement','UnsignedRightConservation','SignedRightFloorConservation','SignedRightRemainderBound','SignedRightTop']:expected.add(f'False-{n}-W{w}')
  for op in ['ShiftLeft','ShiftRight','SignedShiftRight']:
   expected.add(f'Wrong-{op}-W{w}');expected.add(f'Missing-CountBit63-{op}-W{w}')
 expected.update(['Wrong-UnsignedLeftDiscard-W64','Wrong-SignedRightRemainder-W64','Wrong-CountLowComplement6','False-SelectTop64'])
 assert len(expected)==83
 return expected
