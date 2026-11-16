"""
Implementation of the QuickSort algorithm in Python.
"""

def quick_sort(arr):
    """
    Sorts an array in ascending order using QuickSort.
    
    Parameters:
        arr (list): The list of comparable elements to sort.
        
    Returns:
        list: The sorted list.
    """
    # Base case: if array has 0 or 1 elements, it is already sorted
    if len(arr) <= 1:
        return arr
    
    # Choose the middle element as the pivot
    pivot = arr[len(arr) // 2]
    
    # Partition elements into three sublists
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    
    # Recursively sort sublists and concatenate results
    return quick_sort(left) + middle + quick_sort(right)

# Example usage:
if __name__ == "__main__":
    sample_list = [64, 34, 25, 12, 22, 11, 90]
    sorted_result = quick_sort(sample_list)
    print("Sorted array:", sorted_result)
