-- Structure representing a physical frame in memory
structure Frame where
  addr : Nat
  free : Bool
  -- Alignment invariant: page address must be a multiple of 4096 bytes
  is_aligned : addr % 4096 = 0

-- State of the kernel allocator containing all physical frames
structure FrameAllocator where
  frames : List Frame

-- Predicate defining what it means for a frame to be successfully allocated
def is_allocated (alloc : FrameAllocator) (target_addr : Nat) : Prop :=
  Exists (fun f => f ∈ alloc.frames ∧ f.addr = target_addr ∧ f.free = false)

-- Allocation function: finds the first free frame and marks it as occupied
def allocate (alloc : FrameAllocator) : Option (Frame × FrameAllocator) :=
  match alloc.frames.find? (fun f => f.free) with
  | none => none
  | some free_frame =>
    let updated_frames := alloc.frames.map (fun f => if f.addr = free_frame.addr then { f with free := false } else f)
    some (free_frame, { frames := updated_frames })

-- SAFETY THEOREM: If allocation succeeds, the returned frame is guaranteed to be marked as allocated
theorem allocate_marks_as_allocated (alloc : FrameAllocator) (f : Frame) (new_alloc : FrameAllocator) :
  allocate alloc = some (f, new_alloc) → is_allocated new_alloc f.addr := by
  sorry

