import Image from '../../assets/bg.png';


export function PlayBox() {
    return <div className='w-[45%] h-[300px] border-4 border-white text-center rounded-md'>
        <div className='h-[75%]'>
            <img
                className='h-full w-full object-cover'
                src={Image}
                alt='bg'
            />
        </div>
        <div className='p-3'>
            <span>The Box</span>
        </div>
    </div>;
}